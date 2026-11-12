import { auth } from '@/auth';
export const runtime = 'nodejs';
// Owner-authorized free early access applies only to Pixa apps, never builder billing.
export async function GET() {
  const user = (await auth())?.user;
  return Response.json({hasAccess: Boolean(user?.id), accessMode: 'free-early-access', expiresAt: null, cardRequired: false}, {status: user?.id ? 200 : 401, headers:{'Cache-Control':'no-store'}});
}
