import { notFound } from 'next/navigation';
import { ProductDetail } from '@/components/pixa/Catalog';
import { products } from '@/lib/pixa-products';
export const dynamicParams=false;
export function generateStaticParams(){return products.map(p=>({slug:p.id}))}
export async function generateMetadata({params}:{params:Promise<{slug:string}>}){const {slug}=await params;const p=products.find(p=>p.id===slug);return p?{title:`${p.name} — ${p.category}`,description:p.summary}:{};}
export default async function Product({params}:{params:Promise<{slug:string}>}){const {slug}=await params;const product=products.find(p=>p.id===slug);if(!product)notFound();return <ProductDetail product={product}/>}
