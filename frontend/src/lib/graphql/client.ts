import { GraphQLClient } from 'graphql-request';

const apiUrl = import.meta.env.VITE_GRAPHQL_API_URL;


export const graphqlClient = new GraphQLClient(apiUrl, {
    headers: {
        'Content-Type': 'application/json',
    },
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