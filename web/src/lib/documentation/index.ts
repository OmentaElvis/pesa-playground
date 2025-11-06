import type { ComponentType } from 'svelte';

export interface DocArticle {
  slug: string;
  title: string;
  order?: number;
  summary?: string;
  component: ComponentType;
}

const modules = import.meta.glob('./*.svx', { eager: true });

const articles: DocArticle[] = Object.entries(modules).map(([path, module]) => {
  const slug = path.replace('./', '').replace('.svx', '');
  const metadata = (module as any).metadata || {};
  return {
    slug,
    title: metadata.title || slug,
    order: metadata.order,
    summary: metadata.summary,
    component: (module as any).default,
  };
});

export function getArticles(): DocArticle[] {
  return articles.sort((a, b) => {
    const orderA = a.order ?? Infinity;
    const orderB = b.order ?? Infinity;
    if (orderA !== orderB) {
      return orderA - orderB;
    }
    return a.title.localeCompare(b.title);
  });
}

export function getArticle(slug: string): DocArticle | undefined {
  return articles.find(article => article.slug === slug);
}
