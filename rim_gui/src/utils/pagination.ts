import type { Ref } from 'vue'
import { computed, ref } from 'vue'

interface PaginationOptions<T = any> {
  data: T[]
  current?: number
  size?: number
  tagsGetter?: (item: T) => string[]
  tags?: string[]
  /** filter类型 交集或并集 */
  filterType?: 'intersection' | 'union'
}

export function usePagination<T = any>(options: PaginationOptions<T>) {
  const allData = ref(options.data) as Ref<T[]>
  const current = ref(options.current || 1)
  const size = ref(options.size || 10)
  const tags = ref(options.tags || [])

  const {
    tagsGetter = (item) => {
      const tags = (item as any)?.frontmatter?.tags
      if (typeof tags === 'string')
        return [tags]
      return Array.isArray(tags) ? tags : []
    },
  } = options

  const { filterType = 'intersection' } = options

  const tagsList = computed(() => Array.isArray(tags.value) ? tags.value : [tags.value])

  const filteredData = computed(() => allData.value.filter((item) => {
    const itemTags = tagsGetter(item)
    if (tagsList.value.length && itemTags.length) {
      return filterType === 'intersection' ?
        tagsList.value.some(tag => itemTags.includes(tag)) :
        tagsList.value.every(tag => itemTags.includes(tag))
    }

    return true
  }))

  const total = computed(() => filteredData.value.length)

  const list = computed(() => {
    const start = (current.value - 1) * size.value
    const end = start + size.value
    return filteredData.value.slice(start, end)
  })

  return {
    current,
    size,
    tags,
    total,
    list,
    allData,
  }
}
