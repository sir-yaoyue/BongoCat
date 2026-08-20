<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'

import { useGoldStore } from '@/stores/gold'

const { t } = useI18n()
const goldStore = useGoldStore()

const GOLD_COLOR = '#f5c518'
const UP_COLOR = '#ff4d4f'
const DOWN_COLOR = '#52c41a'

const priceText = computed(() => {
  const price = goldStore.price

  if (!price) return '--'

  return price.price.toFixed(2)
})

const changeText = computed(() => {
  const price = goldStore.price

  if (!price) return ''

  const sign = price.change > 0 ? '+' : ''

  return `${sign}${price.change.toFixed(2)} (${sign}${price.changePercent.toFixed(2)}%)`
})

const changeColor = computed(() => {
  const change = goldStore.price?.change ?? 0

  if (change > 0) return UP_COLOR
  if (change < 0) return DOWN_COLOR

  return 'rgba(255, 255, 255, 0.85)'
})
</script>

<template>
  <div class="flex items-center gap-1 whitespace-nowrap bg-black/50 px-2 py-0.5 backdrop-blur-sm rounded-full">
    <span class="text-[5vw] text-white font-bold">{{ t('pages.main.gold.label') }}</span>

    <span
      class="text-[5vw] font-bold"
      :style="{ color: GOLD_COLOR }"
    >
      {{ priceText }}
    </span>

    <span class="text-[5vw] text-white font-bold">{{ t('pages.main.gold.unit') }}</span>

    <span
      class="text-[4.5vw] font-bold"
      :style="{ color: changeColor }"
    >
      {{ changeText }}
    </span>
  </div>
</template>
