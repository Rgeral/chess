import { GraphQLClient } from 'graphql-request';

// Resolve endpoint: prefer VITE_GRAPHQL_API_URL, else "/graphql" proxied by Nginx.
// Ensure absolute URL at runtime to satisfy environments that require it.
const raw = import.meta.env.VITE_GRAPHQL_API_URL || '/graphql';
const apiUrl = raw.startsWith('http')
  ? raw
  : (typeof window !== 'undefined'
      ? new URL(raw, window.location.origin).toString()
      : raw);

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
