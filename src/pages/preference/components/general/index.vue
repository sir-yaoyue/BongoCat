<script setup lang="ts">
import { DeleteOutlined, PlusOutlined } from '@antdv-next/icons'
import { disable, enable, isEnabled } from '@tauri-apps/plugin-autostart'
import { Button, Flex, InputNumber, Select, Switch } from 'antdv-next'
import { nanoid } from 'nanoid'
import { computed, watch } from 'vue'
import { useI18n } from 'vue-i18n'

import ProListItem from '@/components/pro-list-item/index.vue'
import ProList from '@/components/pro-list/index.vue'
import { useGeneralStore } from '@/stores/general'
import { useGoldStore } from '@/stores/gold'
import { useStockStore } from '@/stores/stock'
import { isMac, isWindows } from '@/utils/platform'

import Language from './components/language/index.vue'
import MacosPermissions from './components/macos-permissions/index.vue'
import ThemeMode from './components/theme-mode/index.vue'
import WindowsPermissions from './components/windows-permissions/index.vue'

const generalStore = useGeneralStore()
const goldStore = useGoldStore()
const stockStore = useStockStore()
const { t } = useI18n()

const directionOptions = computed(() => [
  { value: 'upper', label: t('pages.preference.general.labels.targetUpper') },
  { value: 'lower', label: t('pages.preference.general.labels.targetLower') },
])

function addTarget() {
  goldStore.targets.push({ id: nanoid(), direction: 'upper', price: 0 })
}

function removeTarget(id: string) {
  goldStore.targets = goldStore.targets.filter(item => item.id !== id)
}

watch(() => generalStore.app.autostart, async (value) => {
  const enabled = await isEnabled()

  if (value && !enabled) {
    return enable()
  }

  if (!value && enabled) {
    disable()
  }
}, { immediate: true })
</script>

<template>
  <MacosPermissions v-if="isMac" />

  <WindowsPermissions v-if="isWindows" />

  <ProList :title="$t('pages.preference.general.labels.appSettings')">
    <ProListItem :title="$t('pages.preference.general.labels.launchOnStartup')">
      <Switch v-model:checked="generalStore.app.autostart" />
    </ProListItem>

    <ProListItem
      :description="$t('pages.preference.general.hints.showTaskbarIcon')"
      :title="$t('pages.preference.general.labels.showTaskbarIcon')"
    >
      <Switch v-model:checked="generalStore.app.taskbarVisible" />
    </ProListItem>

    <ProListItem
      :description="$t('pages.preference.general.hints.showTrayIcon')"
      :title="$t('pages.preference.general.labels.showTrayIcon')"
    >
      <Switch v-model:checked="generalStore.app.trayVisible" />
    </ProListItem>
  </ProList>

  <ProList :title="$t('pages.preference.general.labels.appearanceSettings')">
    <ThemeMode />

    <Language />
  </ProList>

  <ProList :title="$t('pages.preference.general.labels.updateSettings')">
    <ProListItem :title="$t('pages.preference.general.labels.autoCheckUpdate')">
      <Switch v-model:checked="generalStore.update.autoCheck" />
    </ProListItem>
  </ProList>

  <ProList :title="$t('pages.preference.general.labels.goldSettings')">
    <ProListItem
      :description="$t('pages.preference.general.hints.showGoldPrice')"
      :title="$t('pages.preference.general.labels.showGoldPrice')"
    >
      <Switch v-model:checked="goldStore.enabled" />
    </ProListItem>

    <ProListItem
      :description="$t('pages.preference.general.hints.notifyOnGoldPrice')"
      :title="$t('pages.preference.general.labels.notifyOnGoldPrice')"
    >
      <Switch v-model:checked="goldStore.notifyEnabled" />
    </ProListItem>

    <ProListItem
      v-if="goldStore.notifyEnabled"
      :title="$t('pages.preference.general.labels.targetPrices')"
      vertical
    >
      <Flex
        gap="small"
        vertical
      >
        <Flex
          v-for="target in goldStore.targets"
          :key="target.id"
          align="center"
          gap="small"
        >
          <Select
            v-model:value="target.direction"
            class="w-30 shrink-0"
            :options="directionOptions"
          />

          <InputNumber
            v-model:value="target.price"
            :addon-after="$t('pages.main.gold.unit')"
            class="flex-1"
            :min="0"
            :precision="2"
            :step="1"
          />

          <Button
            :aria-label="$t('pages.preference.general.labels.removeTarget')"
            danger
            type="text"
            @click="removeTarget(target.id)"
          >
            <template #icon>
              <DeleteOutlined />
            </template>
          </Button>
        </Flex>

        <Button
          block
          type="dashed"
          @click="addTarget"
        >
          <template #icon>
            <PlusOutlined />
          </template>
          {{ $t('pages.preference.general.labels.addTarget') }}
        </Button>
      </Flex>
    </ProListItem>
  </ProList>

  <ProList :title="$t('pages.preference.general.labels.stockSettings')">
    <ProListItem
      :description="$t('pages.preference.general.hints.showStockIndex')"
      :title="$t('pages.preference.general.labels.showStockIndex')"
    >
      <Switch v-model:checked="stockStore.enabled" />
    </ProListItem>
  </ProList>
</template>
