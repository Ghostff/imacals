import { ref, type Ref } from 'vue';
import {
  wishlistService,
  type WishlistDetail,
  type WishlistSummary,
} from '@/services/wishlist';
import { ApiException } from '@/services/api';

// Module-level singleton so the wishlist page, header badge, and product card share one cache.
const lists: Ref<WishlistSummary[]> = ref<WishlistSummary[]>([]);
const loaded: Ref<boolean> = ref<boolean>(false);
const loading: Ref<boolean> = ref<boolean>(false);

export function useWishlist(): {
  lists: Ref<WishlistSummary[]>;
  loaded: Ref<boolean>;
  loading: Ref<boolean>;
  totalCount: Ref<number>;
  refresh: () => Promise<void>;
  loadOne: (id: string) => Promise<WishlistDetail | null>;
  create: (name: string, description?: string) => Promise<WishlistSummary | null>;
  rename: (id: string, name: string, description?: string | null) => Promise<WishlistSummary | null>;
  remove: (id: string) => Promise<boolean>;
  addItem: (wishlistId: string, productId: string) => Promise<WishlistDetail | null>;
  removeItem: (wishlistId: string, itemId: string) => Promise<WishlistDetail | null>;
} {
  const totalCount: Ref<number> = ref<number>(0);

  function isLoggedIn(): boolean {
    return localStorage.getItem('token') !== null;
  }

  async function refresh(): Promise<void> {
    if (!isLoggedIn()) {
      lists.value = [];
      loaded.value = false;
      totalCount.value = 0;
      return;
    }
    loading.value = true;
    try {
      lists.value = await wishlistService.list();
      totalCount.value = lists.value.reduce<number>((sum, w) => sum + w.item_count, 0);
      loaded.value = true;
    } catch (e: unknown) {
      // 401 means the token is bad — clear local cache but don't throw, the page can still render.
      if (e instanceof ApiException && e.status === 401) {
        lists.value = [];
        totalCount.value = 0;
        loaded.value = true;
        return;
      }
      throw e;
    } finally {
      loading.value = false;
    }
  }

  async function loadOne(id: string): Promise<WishlistDetail | null> {
    try {
      return await wishlistService.show(id);
    } catch {
      return null;
    }
  }

  async function create(name: string, description?: string): Promise<WishlistSummary | null> {
    try {
      const created = await wishlistService.create(name, description);
      await refresh();
      return created;
    } catch {
      return null;
    }
  }

  async function rename(
    id: string,
    name: string,
    description?: string | null,
  ): Promise<WishlistSummary | null> {
    try {
      const updated = await wishlistService.update(id, { name, description });
      await refresh();
      return updated;
    } catch {
      return null;
    }
  }

  async function remove(id: string): Promise<boolean> {
    try {
      await wishlistService.remove(id);
      await refresh();
      return true;
    } catch {
      return false;
    }
  }

  async function addItem(wishlistId: string, productId: string): Promise<WishlistDetail | null> {
    try {
      return await wishlistService.addItem(wishlistId, productId);
    } catch {
      return null;
    }
  }

  async function removeItem(
    wishlistId: string,
    itemId: string,
  ): Promise<WishlistDetail | null> {
    try {
      await wishlistService.removeItem(wishlistId, itemId);
      await refresh();
      return null;
    } catch {
      return null;
    }
  }

  return {
    lists,
    loaded,
    loading,
    totalCount,
    refresh,
    loadOne,
    create,
    rename,
    remove,
    addItem,
    removeItem,
  };
}
