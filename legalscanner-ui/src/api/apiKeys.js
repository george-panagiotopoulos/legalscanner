import client from './client'

export const createApiKey = async (name) => {
  const response = await client.post('/api/v1/api-keys', { name })
  return response.data
}

export const getApiKeys = async () => {
  const response = await client.get('/api/v1/api-keys')
  return response.data
}

export const deleteApiKey = async (keyId) => {
  await client.delete(`/api/v1/api-keys/${keyId}`)
}
