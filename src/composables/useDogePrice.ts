import { invoke } from '@tauri-apps/api/core'
import { error } from '@tauri-apps/plugin-log'
import { onUnmounted, watch } from 'vue'

import type { DogePrice } from '@/stores/doge'

import { INVOKE_KEY } from '@/constants'
import { useDogeStore } from '@/stores/doge'

const REFRESH_INTERVAL = 10_000

export function useDogePrice() {
  const dogeStore = useDogeStore()
  let timer: ReturnType<typeof setInterval> | undefined

  const fetchPrice = async () => {
    dogeStore.loading = true

    try {
      dogeStore.price = await invoke<DogePrice>(INVOKE_KEY.FETCH_DOGE_PRICE)
    } catch (err) {
      error(String(err))
    } finally {
      dogeStore.loading = false
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

  watch(() => dogeStore.enabled, (value) => {
    value ? start() : stop()
  }, { immediate: true })

  onUnmounted(stop)

  return {
    fetchPrice,
    start,
    stop,
  }
}
