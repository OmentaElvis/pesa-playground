import { getArticles } from '$lib/documentation';

export async function load() {
	return {
		articles: getArticles()
	};
}
