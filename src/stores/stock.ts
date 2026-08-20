import { defineStore } from 'pinia'
import { ref } from 'vue'

export interface StockIndex {
  name: string
  price: number
  change: number
  changePercent: number
}

export const useStockStore = defineStore('stock', () => {
  const enabled = ref(false)
  const price = ref<StockIndex>()
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
