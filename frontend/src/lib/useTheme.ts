import { useDark, useToggle } from '@vueuse/core'

export const isDark = useDark({
    selector: 'html',
    attribute: 'class',
    valueDark: 'dark',
    valueLight: '',
})
export const toggleDark = useToggle(isDark)
