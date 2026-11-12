import { auth } from '@/auth';
import { redirect } from 'next/navigation';
import Brand from '@/components/auth/Brand';
import AccountForm from '@/components/auth/AccountForm';
export const metadata = { title: 'Get Pixa free' };
export default async function StartPage() {
  if ((await auth())?.user?.id) redirect('/account');
  return <main className="brand-shell brand-auth"><section className="brand-card brand-auth-card"><Brand/><p className="brand-eyebrow mt-8">PIXA FREE EARLY ACCESS</p><h1 className="brand-title mt-4">Your apps. Free to explore.</h1><p className="mt-4 mb-6 text-[#AAA9CD]">Create one account to use released Pixa apps. No credit card, subscription or scheduled expiration. More apps join as they launch.</p><AccountForm callbackUrl="/account"/><p className="mt-6 text-sm text-[#9AAAC0]"><a href="/products">Explore the apps →</a></p></section></main>;
}
