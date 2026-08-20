import { invoke } from '@tauri-apps/api/core'
import { error } from '@tauri-apps/plugin-log'
import { isPermissionGranted, requestPermission, sendNotification } from '@tauri-apps/plugin-notification'
import { onUnmounted, watch } from 'vue'
import { useI18n } from 'vue-i18n'

import type { GoldPrice } from '@/stores/gold'

import { INVOKE_KEY } from '@/constants'
import { useGoldStore } from '@/stores/gold'

const REFRESH_INTERVAL = 10_000

type NotifyDirection = 'upper' | 'lower'

export function useGoldPrice() {
  const goldStore = useGoldStore()
  const { t } = useI18n()

  let timer: ReturnType<typeof setInterval> | undefined
  let previousPrice: number | undefined

  const ensureNotificationPermission = async () => {
    if (await isPermissionGranted()) return true

    const permission = await requestPermission()

    return permission === 'granted'
  }

  const notify = async (direction: NotifyDirection, price: number, target: number) => {
    try {
      if (!(await ensureNotificationPermission())) return

      const priceText = price.toFixed(2)
      const targetText = target.toFixed(2)

      const title = direction === 'upper'
        ? t('pages.main.gold.notify.upperTitle')
        : t('pages.main.gold.notify.lowerTitle')

      const body = direction === 'upper'
        ? t('pages.main.gold.notify.upperBody', { price: priceText, target: targetText })
        : t('pages.main.gold.notify.lowerBody', { price: priceText, target: targetText })

      sendNotification({ title, body })
    } catch (err) {
      error(String(err))
    }
  }

  const checkTarget = (price: number) => {
    if (!goldStore.notifyEnabled || previousPrice === undefined) return

    // 仅在价格“越过”点位时触发一次，避免持续高于/低于阈值时反复提醒
    for (const target of goldStore.targets) {
      const targetPrice = target.price

      if (typeof targetPrice !== 'number' || Number.isNaN(targetPrice)) continue

      if (target.direction === 'upper') {
        if (previousPrice < targetPrice && price >= targetPrice) {
          void notify('upper', price, targetPrice)
        }
      } else if (previousPrice > targetPrice && price <= targetPrice) {
        void notify('lower', price, targetPrice)
      }
    }
  }

  const fetchPrice = async () => {
    goldStore.loading = true

    try {
      const price = await invoke<GoldPrice>(INVOKE_KEY.FETCH_GOLD_PRICE)

      checkTarget(price.price)

      previousPrice = price.price
      goldStore.price = price
    } catch (err) {
      error(String(err))
    } finally {
      goldStore.loading = false
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
    previousPrice = void 0
  }

  watch(() => goldStore.enabled || goldStore.notifyEnabled, (value) => {
    value ? start() : stop()
  }, { immediate: true })

  onUnmounted(stop)

  return {
    fetchPrice,
    start,
    stop,
  }
}
