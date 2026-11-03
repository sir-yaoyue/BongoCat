<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'

import { useDogeStore } from '@/stores/doge'

const { t } = useI18n()
const dogeStore = useDogeStore()

const DOGE_COLOR = '#f39c12'
const UP_COLOR = '#ff6b6b'
const DOWN_COLOR = '#51cf66'

const priceText = computed(() => {
  const price = dogeStore.price

  if (!price) return '--'

  return price.price.toFixed(4)
})

const changeText = computed(() => {
  const price = dogeStore.price

  if (!price) return ''

  const sign = price.change > 0 ? '+' : ''

  return `${sign}${price.change.toFixed(4)} (${sign}${price.changePercent.toFixed(2)}%)`
})

const changeColor = computed(() => {
  const change = dogeStore.price?.change ?? 0

  if (change > 0) return UP_COLOR
  if (change < 0) return DOWN_COLOR

  return 'rgba(255, 255, 255, 0.9)'
})
</script>

<template>
  <div class="price-pill">
    <span class="text-[5.2vw] text-white font-bold">{{ t('pages.main.doge.label') }}</span>

    <span
      class="text-[5.2vw] font-extrabold"
      :style="{ color: DOGE_COLOR }"
    >
      {{ priceText }}
    </span>

    <span class="text-[5.2vw] text-white font-bold">{{ t('pages.main.doge.unit') }}</span>

    <span
      class="text-[4.5vw] font-semibold"
      :style="{ color: changeColor }"
    >
      {{ changeText }}
    </span>
  </div>
</template>
