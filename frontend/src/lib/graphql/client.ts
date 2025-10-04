import { GraphQLClient } from 'graphql-request';

// Utilise la variable d'env si définie, sinon fallback sur un chemin relatif
// qui sera proxyé par Nginx vers le backend (/graphql -> backend:8080/graphql)
const apiUrl = import.meta.env.VITE_GRAPHQL_API_URL || '/graphql';

export const graphqlClient = new GraphQLClient(apiUrl, {
	headers: {
		'Content-Type': 'application/json'
	}
});

// Fonction helper pour les requêtes
export async function executeGraphQL<T = any>(query: string, variables?: any): Promise<T> {
	try {
		return await graphqlClient.request<T>(query, variables);
	} catch (error) {
		console.error('GraphQL Error:', error);
		throw error;
	}
}
