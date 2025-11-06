import { getArticle } from '$lib/documentation';
import { error, type ServerLoad } from '@sveltejs/kit';

export const load: ServerLoad = async({ params }) => {
  const article = getArticle(params.slug || "");

  if (!article) {
    error(404, 'Not found');
  }

  return {
    article
  };
}
