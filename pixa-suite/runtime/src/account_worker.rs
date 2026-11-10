//! One worker serializes HTTP and secure storage. Disconnect is acknowledged only after deletion.
use crate::{
    account::{self, Answer, DeviceToken},
    credentials::{CredentialStore, NativeStore},
};
use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
    mpsc,
};

pub struct Checked {
    pub token: DeviceToken,
    pub answer: Answer,
    pub warning: Option<String>,
    pub verified_at: std::time::Instant,
}
pub enum Reply {
    Checked {
        generation: u64,
        result: Result<Checked, String>,
    },
    Forgotten {
        generation: u64,
        result: Result<(), String>,
    },
}
enum Command {
    Check { generation: u64, token: DeviceToken },
    Forget { generation: u64 },
    Stop,
}
pub struct Worker {
    tx: mpsc::SyncSender<Command>,
    rx: mpsc::Receiver<Reply>,
    generation: Arc<AtomicU64>,
}
impl Worker {
    pub fn native(product: &str) -> Result<Self, String> {
        Self::with(NativeStore::new(product)?, account::check)
    }
    pub fn with<S, C>(mut store: S, checker: C) -> Result<Self, String>
    where
        S: CredentialStore,
        C: Fn(&DeviceToken) -> Result<Answer, String> + Send + 'static,
    {
        let (tx, requests) = mpsc::sync_channel(4);
        let (replies, rx) = mpsc::channel();
        let generation = Arc::new(AtomicU64::new(0));
        let current = generation.clone();
        std::thread::Builder::new().name("Pixa account and secure storage".into()).spawn(move||{
            while let Ok(command)=requests.recv(){match command{
                Command::Stop=>break,
                Command::Forget{generation}=>{let result=store.delete();let _=replies.send(Reply::Forgotten{generation,result});}
                Command::Check{generation,mut token}=>{
                    let mut warning=None;
                    if token.is_empty(){match store.load(){Ok(Some(saved))=>token=saved,Ok(None)=>{},Err(_)=>warning=Some("Secure storage is unavailable; sign-in lasts for this session.".into())}}
                    let verified_at=std::time::Instant::now();
                    let result=checker(&token).and_then(|answer|{
                        answer.validate()?;
                        if let Some(issued)=answer.issued_token()?{token=issued;}
                        if generation!=current.load(Ordering::Acquire){return Err("Ignored an outdated account check.".into());}
                        if !token.is_empty() && store.save(&token).is_err(){warning=Some("Sign-in is active only for this session because secure storage could not save it.".into());}
                        Ok(Checked{token,answer,warning,verified_at})
                    });
                    let _=replies.send(Reply::Checked{generation,result});
                }
            }}
        }).map_err(|_|"Could not start the account worker.")?;
        Ok(Self { tx, rx, generation })
    }
    pub fn check(&self, generation: u64, token: DeviceToken) -> Result<(), String> {
        self.generation.store(generation, Ordering::Release);
        self.tx
            .try_send(Command::Check { generation, token })
            .map_err(|_| "The account worker stopped.".into())
    }
    pub fn forget(&self, generation: u64) -> Result<(), String> {
        self.generation.store(generation, Ordering::Release);
        self.tx
            .try_send(Command::Forget { generation })
            .map_err(|_| "The account worker stopped before disconnecting.".into())
    }
    pub fn poll(&self) -> Result<Option<Reply>, String> {
        match self.rx.try_recv() {
            Ok(reply) => Ok(Some(reply)),
            Err(mpsc::TryRecvError::Empty) => Ok(None),
            Err(_) => Err("The account worker stopped.".into()),
        }
    }
}
impl Drop for Worker {
    fn drop(&mut self) {
        let _ = self.tx.try_send(Command::Stop);
    }
}
#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;
    use std::{sync::Mutex, time::Duration};
    #[derive(Clone, Default)]
    struct Memory(Arc<Mutex<Option<DeviceToken>>>);
    impl CredentialStore for Memory {
        fn load(&mut self) -> Result<Option<DeviceToken>, String> {
            Ok(self.0.lock().map_err(|_| "lock")?.clone())
        }
        fn save(&mut self, t: &DeviceToken) -> Result<(), String> {
            *self.0.lock().map_err(|_| "lock")? = Some(t.clone());
            Ok(())
        }
        fn delete(&mut self) -> Result<(), String> {
            *self.0.lock().map_err(|_| "lock")? = None;
            Ok(())
        }
    }
    fn answer() -> Answer {
        serde_json::from_value(
            serde_json::json!({"token":"a".repeat(64),"code":"FIXTURE","access":false}),
        )
        .unwrap()
    }
    #[test]
    fn disconnect_after_pending_check_cannot_leave_old_sign_in() {
        let memory = Memory::default();
        let copy = memory.clone();
        let (entered, waiting) = mpsc::channel();
        let (go, resume) = mpsc::channel();
        let resume = Mutex::new(resume);
        let worker = Worker::with(memory, move |_| {
            entered.send(()).map_err(|_| "closed")?;
            resume
                .lock()
                .map_err(|_| "lock")?
                .recv_timeout(Duration::from_secs(3))
                .map_err(|_| "timeout")?;
            Ok(answer())
        })
        .unwrap();
        worker.check(0, DeviceToken::default()).unwrap();
        waiting.recv_timeout(Duration::from_secs(3)).unwrap();
        worker.forget(1).unwrap();
        go.send(()).unwrap();
        let _ = worker.rx.recv_timeout(Duration::from_secs(3)).unwrap();
        match worker.rx.recv_timeout(Duration::from_secs(3)).unwrap() {
            Reply::Forgotten {
                generation: 1,
                result: Ok(()),
            } => {}
            _ => panic!("disconnect was not acknowledged"),
        }
        assert!(copy.0.lock().unwrap().is_none());
    }
    #[test]
    fn disconnect_waits_for_inflight_secure_write_then_deletes_it() {
        struct Blocking {
            memory: Memory,
            entered: mpsc::Sender<()>,
            resume: mpsc::Receiver<()>,
        }
        impl CredentialStore for Blocking {
            fn load(&mut self) -> Result<Option<DeviceToken>, String> {
                self.memory.load()
            }
            fn save(&mut self, t: &DeviceToken) -> Result<(), String> {
                self.entered.send(()).map_err(|_| "closed")?;
                self.resume
                    .recv_timeout(Duration::from_secs(3))
                    .map_err(|_| "timeout")?;
                self.memory.save(t)
            }
            fn delete(&mut self) -> Result<(), String> {
                self.memory.delete()
            }
        }
        let memory = Memory::default();
        let copy = memory.clone();
        let (entered, waiting) = mpsc::channel();
        let (go, resume) = mpsc::channel();
        let worker = Worker::with(
            Blocking {
                memory,
                entered,
                resume,
            },
            |_| Ok(answer()),
        )
        .unwrap();
        worker.check(0, DeviceToken::default()).unwrap();
        waiting.recv_timeout(Duration::from_secs(3)).unwrap();
        worker.forget(1).unwrap();
        go.send(()).unwrap();
        let _ = worker.rx.recv_timeout(Duration::from_secs(3)).unwrap();
        match worker.rx.recv_timeout(Duration::from_secs(3)).unwrap() {
            Reply::Forgotten {
                generation: 1,
                result: Ok(()),
            } => {}
            _ => panic!("deletion was not acknowledged"),
        }
        assert!(copy.0.lock().unwrap().is_none());
    }
    #[test]
    fn secure_store_failure_has_no_plaintext_fallback() {
        struct Failed;
        impl CredentialStore for Failed {
            fn load(&mut self) -> Result<Option<DeviceToken>, String> {
                Err("unavailable".into())
            }
            fn save(&mut self, _: &DeviceToken) -> Result<(), String> {
                Err("unavailable".into())
            }
            fn delete(&mut self) -> Result<(), String> {
                Err("unavailable".into())
            }
        }
        let worker = Worker::with(Failed, |_| Ok(answer())).unwrap();
        worker.check(0, DeviceToken::default()).unwrap();
        match worker.rx.recv_timeout(Duration::from_secs(3)).unwrap() {
            Reply::Checked {
                result: Ok(checked),
                ..
            } => assert!(checked.warning.is_some()),
            _ => panic!("fixture check failed"),
        }
        worker.forget(1).unwrap();
        match worker.rx.recv_timeout(Duration::from_secs(3)).unwrap() {
            Reply::Forgotten { result: Err(_), .. } => {}
            _ => panic!("failed deletion must not be reported as success"),
        }
    }
}
