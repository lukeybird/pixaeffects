import { CatalogShell, ProductGrid } from '@/components/pixa/Catalog';
import s from '@/components/pixa/catalog.module.css';
export const metadata={title:'All Pixa apps',description:'Explore every Pixa desktop app and its current release and download status.'};
export default function Products(){return <CatalogShell><div id="main-content"><section className={s.hero}><p className={s.eyebrow}>THE COMPLETE FAMILY</p><h1>A workspace<br/><span>for every idea.</span></h1><p className={s.lead}>Images, films, layouts, documents, sound. Find the app built around your kind of work.</p></section><ProductGrid/></div></CatalogShell>}
