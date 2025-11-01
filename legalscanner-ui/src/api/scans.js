import client from './client'

export const createScan = async (gitUrl, gitToken = null) => {
  const payload = { git_url: gitUrl }
  if (gitToken) {
    payload.git_token = gitToken
  }
  const response = await client.post('/api/v1/scans', payload)
  return response.data
}

export const getScans = async () => {
  const response = await client.get('/api/v1/scans')
  return response.data
}

export const getScan = async (scanId) => {
  const response = await client.get(`/api/v1/scans/${scanId}`)
  return response.data
}

export const getScanResults = async (scanId) => {
  const response = await client.get(`/api/v1/scans/${scanId}/results`)
  return response.data
}

export const deleteScan = async (scanId) => {
  await client.delete(`/api/v1/scans/${scanId}`)
}

export const deleteAllScans = async () => {
  const response = await client.delete('/api/v1/scans')
  return response.data
}
