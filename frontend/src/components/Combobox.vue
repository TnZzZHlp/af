<script setup lang="ts">
import type { HTMLAttributes } from "vue";
import { ref, computed } from "vue";
import { Check, ChevronsUpDown, Loader2 } from "lucide-vue-next";
import { cn } from "@/lib/utils";
import { Button } from "@/components/ui/button";
import {
  Command,
  CommandEmpty,
  CommandGroup,
  CommandInput,
  CommandItem,
  CommandList,
} from "@/components/ui/command";
import {
  Popover,
  PopoverContent,
  PopoverTrigger,
} from "@/components/ui/popover";

export interface ComboboxOption {
  value: string | number;
  label: string;
}

const props = defineProps<{
  modelValue?: string | number | null;
  options: ComboboxOption[];
  placeholder?: string;
  searchPlaceholder?: string;
  emptyText?: string;
  disabled?: boolean;
  loading?: boolean;
  unselectable?: boolean;
  class?: HTMLAttributes["class"];
}>();

const emit = defineEmits<{
  (e: "update:modelValue", value: string | number | null | undefined): void;
  (e: "select", option: ComboboxOption): void;
}>();

const open = ref(false);

const selectedLabel = computed(() => {
  const option = props.options.find((o) => o.value === props.modelValue);
  return option ? option.label : props.placeholder || "Select option...";
});

function handleSelect(option: ComboboxOption) {
  if (props.unselectable && props.modelValue === option.value) {
    emit("update:modelValue", null);
  } else {
    emit("update:modelValue", option.value);
  }
  emit("select", option);
  open.value = false;
}
</script>

<template>
  <Popover v-model:open="open">
    <PopoverTrigger as-child>
      <Button
        variant="outline"
        role="combobox"
        :aria-expanded="open"
        :class="cn('justify-between w-full font-normal', !modelValue && 'text-muted-foreground', props.class)"
        :disabled="disabled || loading"
      >
        <span v-if="loading" class="flex items-center gap-2">
          <Loader2 class="h-4 w-4 animate-spin" />
          Loading...
        </span>
        <span v-else class="truncate">
          {{ selectedLabel }}
        </span>
        <ChevronsUpDown class="ml-2 h-4 w-4 shrink-0 opacity-50" />
      </Button>
    </PopoverTrigger>
    <PopoverContent class="w-[--radix-popover-trigger-width] p-0">
      <Command>
        <CommandInput class="h-9" :placeholder="searchPlaceholder || 'Search...'" />
        <CommandList>
          <CommandEmpty>{{ emptyText || 'No results found.' }}</CommandEmpty>
          <CommandGroup>
            <CommandItem
              v-for="option in options"
              :key="option.value"
              :value="String(option.label)" 
              @select="handleSelect(option)"
            >
              <span class="truncate">{{ option.label }}</span>
              <Check
                :class="
                  cn(
                    'ml-auto h-4 w-4',
                    modelValue === option.value ? 'opacity-100' : 'opacity-0'
                  )
                "
              />
            </CommandItem>
          </CommandGroup>
        </CommandList>
      </Command>
    </PopoverContent>
  </Popover>
</template>
