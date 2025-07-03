import { GraphQLClient } from 'graphql-request';

export const graphqlClient = new GraphQLClient('http://localhost:8080/graphql', {
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