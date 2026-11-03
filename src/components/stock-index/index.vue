<script setup lang="ts">
import { computed } from 'vue'

import { useStockStore } from '@/stores/stock'

const stockStore = useStockStore()

const UP_COLOR = '#ff6b6b'
const DOWN_COLOR = '#51cf66'

const priceText = computed(() => {
  const price = stockStore.price

  if (!price) return '--'

  return price.price.toFixed(2)
})

const changeText = computed(() => {
  const price = stockStore.price

  if (!price) return ''

  const sign = price.change > 0 ? '+' : ''

  return `${sign}${price.change.toFixed(2)} (${sign}${price.changePercent.toFixed(2)}%)`
})

const changeColor = computed(() => {
  const change = stockStore.price?.change ?? 0

  if (change > 0) return UP_COLOR
  if (change < 0) return DOWN_COLOR

  return 'rgba(255, 255, 255, 0.9)'
})
</script>

<template>
  <div class="price-pill">
    <span class="text-[5.2vw] text-white font-bold">{{ $t('pages.main.stock.label') }}</span>

    <span class="text-[5.2vw] text-white font-extrabold">
      {{ priceText }}
    </span>

    <span
      class="text-[4.5vw] font-semibold"
      :style="{ color: changeColor }"
    >
      {{ changeText }}
    </span>
  </div>
</template>
