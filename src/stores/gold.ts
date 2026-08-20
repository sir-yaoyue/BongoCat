import { defineStore } from 'pinia'
import { ref } from 'vue'

export interface GoldPrice {
  name: string
  price: number
  change: number
  changePercent: number
  time: string
}

export type GoldTargetDirection = 'upper' | 'lower'

export interface GoldTarget {
  id: string
  direction: GoldTargetDirection
  price: number
}

export const useGoldStore = defineStore('gold', () => {
  const enabled = ref(true)
  const price = ref<GoldPrice>()
  const loading = ref(false)
  const notifyEnabled = ref(false)
  const targets = ref<GoldTarget[]>([])

  return {
    enabled,
    price,
    loading,
    notifyEnabled,
    targets,
  }
}, {
  tauri: {
    filterKeys: ['price', 'loading'],
  },
})
