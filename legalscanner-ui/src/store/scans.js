import { defineStore } from 'pinia'
import { ref } from 'vue'
import * as scansApi from '@/api/scans'

export const useScansStore = defineStore('scans', () => {
  const scans = ref([])
  const currentScan = ref(null)
  const currentResults = ref(null)
  const loading = ref(false)
  const loadingResults = ref(false)
  const error = ref(null)

  const fetchScans = async () => {
    loading.value = true
    error.value = null
    try {
      scans.value = await scansApi.getScans()
    } catch (e) {
      error.value = e.response?.data?.error || e.message
      throw e
    } finally {
      loading.value = false
    }
  }

  const createScan = async (gitUrl, gitToken = null) => {
    loading.value = true
    error.value = null
    try {
      const scan = await scansApi.createScan(gitUrl, gitToken)
      scans.value.unshift(scan)
      return scan
    } catch (e) {
      error.value = e.response?.data?.error || e.message
      throw e
    } finally {
      loading.value = false
    }
  }

  const fetchScan = async (scanId) => {
    loading.value = true
    error.value = null
    try {
      currentScan.value = await scansApi.getScan(scanId)
    } catch (e) {
      error.value = e.response?.data?.error || e.message
      throw e
    } finally {
      loading.value = false
    }
  }

  const fetchResults = async (scanId) => {
    loadingResults.value = true
    error.value = null
    try {
      currentResults.value = await scansApi.getScanResults(scanId)
    } catch (e) {
      error.value = e.response?.data?.error || e.message
      throw e
    } finally {
      loadingResults.value = false
    }
  }

  const deleteScan = async (scanId) => {
    loading.value = true
    error.value = null
    try {
      await scansApi.deleteScan(scanId)
      scans.value = scans.value.filter(s => s.scan_id !== scanId)
    } catch (e) {
      error.value = e.response?.data?.error || e.message
      throw e
    } finally {
      loading.value = false
    }
  }

  return {
    scans,
    currentScan,
    currentResults,
    loading,
    loadingResults,
    error,
    fetchScans,
    createScan,
    fetchScan,
    fetchResults,
    deleteScan
  }
})
