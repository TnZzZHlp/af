<script setup lang="ts">
import { VisTooltip } from "@unovis/vue"
import { type Component } from "vue"
import { useChart, componentToString } from "."

const props = withDefaults(defineProps<{
  content: Component
}>(), {})

const { config } = useChart()

const template = componentToString(config.value, props.content as any)

// Define SelectorType locally matching unovis css classes
const SelectorType = {
  Point: '.unovis-symbol',
  Bar: '.unovis-bar',
  Area: '.unovis-area',
  Scatter: '.unovis-scatter', // This might be .unovis-symbol too
  Donut: '.unovis-donut-segment',
  Timeline: '.unovis-timeline',
  Crosshair: '.unovis-crosshair',
}
</script>

<template>
  <VisTooltip
    :triggers="{
      [SelectorType.Point]: template,
      [SelectorType.Bar]: template,
      [SelectorType.Area]: template,
      [SelectorType.Scatter]: template,
      [SelectorType.Donut]: template,
      [SelectorType.Timeline]: template,
      [SelectorType.Crosshair]: template,
    }"
  />
</template>
