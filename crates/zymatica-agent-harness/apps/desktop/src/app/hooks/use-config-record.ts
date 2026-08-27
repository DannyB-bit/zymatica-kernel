import { useQuery } from '@tanstack/react-query'

import { getZymaticaConfigRecord } from '@/zymatica'
import { queryClient, writeCache } from '@/lib/query-client'
import type { ZymaticaConfigRecord } from '@/types/zymatica'

// One shared cache for the whole profile config record (`GET /api/config`).
// Every settings surface (MCP, model, config) reads and writes through this key
// so a save in one shows in the others, and revisiting a tab paints the cache
// instead of blanking on a fresh fetch.
//
// Distinct from session/hooks/use-zymatica-config.ts, which is side-effecting —
// it pushes personality/cwd/voice/… into the session stores for live chat.
export const ZYMATICA_CONFIG_KEY = ['zymatica-config-record'] as const

// staleTime 0 → serve cache instantly, background-revalidate on every mount.
export const useZymaticaConfigRecord = () =>
  useQuery({ queryKey: ZYMATICA_CONFIG_KEY, queryFn: getZymaticaConfigRecord, staleTime: 0 })

export const setZymaticaConfigCache = writeCache<ZymaticaConfigRecord>(ZYMATICA_CONFIG_KEY)

export const invalidateZymaticaConfig = () => queryClient.invalidateQueries({ queryKey: ZYMATICA_CONFIG_KEY })
