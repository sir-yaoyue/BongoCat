import { defineStore } from 'pinia'
import { ref } from 'vue'

export interface DogePrice {
  name: string
  price: number
  change: number
  changePercent: number
  time: string
}

export const useDogeStore = defineStore('doge', () => {
  const enabled = ref(true)
  const price = ref<DogePrice>()
  const loading = ref(false)

  return {
    enabled,
    price,
    loading,
  }
}, {
  tauri: {
    filterKeys: ['price', 'loading'],
  },
})
