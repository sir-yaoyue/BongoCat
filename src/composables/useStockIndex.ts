import { invoke } from '@tauri-apps/api/core'
import { error } from '@tauri-apps/plugin-log'
import { onUnmounted, watch } from 'vue'

import type { StockIndex } from '@/stores/stock'

import { INVOKE_KEY } from '@/constants'
import { useStockStore } from '@/stores/stock'

const REFRESH_INTERVAL = 10_000

export function useStockIndex() {
  const stockStore = useStockStore()
  let timer: ReturnType<typeof setInterval> | undefined

  const fetchPrice = async () => {
    stockStore.loading = true

    try {
      stockStore.price = await invoke<StockIndex>(INVOKE_KEY.FETCH_STOCK_INDEX)
    } catch (err) {
      error(String(err))
    } finally {
      stockStore.loading = false
    }
  }

  const start = () => {
    stop()

    fetchPrice()

    timer = setInterval(fetchPrice, REFRESH_INTERVAL)
  }

  const stop = () => {
    if (!timer) return

    clearInterval(timer)

    timer = void 0
  }

  watch(() => stockStore.enabled, (value) => {
    value ? start() : stop()
  }, { immediate: true })

  onUnmounted(stop)

  return {
    fetchPrice,
    start,
    stop,
  }
}
