// Service for managing customer delivery addresses.
// Stored per authenticated user so repeat customers can select addresses at checkout and on their dashboard.

export interface CustomerAddress {
  id: string;
  user_id?: string;
  label: string; // e.g. "Main Commercial Store", "Warehouse Annex", "Residential"
  recipient_name: string;
  phone: string;
  street: string;
  landmark?: string;
  city: string;
  state: string;
  is_default: boolean;
  instructions?: string;
  created_at: string;
}

export type CreateAddressInput = Omit<CustomerAddress, 'id' | 'created_at'>;
export type UpdateAddressInput = Partial<Omit<CustomerAddress, 'id' | 'created_at'>>;

export interface AddressUserDefaults {
  name?: string;
  phone?: string;
}

const STORAGE_PREFIX = 'imacals_addresses_';

function getStorageKey(userId?: string): string {
  return `${STORAGE_PREFIX}${userId || 'guest'}`;
}

function notifyAddressChanged(): void {
  if (typeof window !== 'undefined') {
    window.dispatchEvent(new CustomEvent('imacals:address-changed'));
  }
}

export const customerAddressService = {
  async listAddresses(userId?: string, defaults?: AddressUserDefaults): Promise<CustomerAddress[]> {
    const key = getStorageKey(userId);
    const stored = localStorage.getItem(key);
    if (!stored) {
      // Seed an initial live address customized to the user if known
      const recipientName = defaults?.name?.trim() || 'Primary Contact';
      const recipientPhone = defaults?.phone?.trim() || '';

      const initialAddresses: CustomerAddress[] = [
        {
          id: `addr-${Date.now()}-1`,
          user_id: userId,
          label: 'Main Commercial Store',
          recipient_name: recipientName,
          phone: recipientPhone,
          street: '14 Faulks Road',
          landmark: 'Opposite Ariaria International Market Gate 2',
          city: 'Aba',
          state: 'Abia State',
          is_default: true,
          instructions: 'Call on arrival before offloading.',
          created_at: new Date(Date.now() - 30 * 86400000).toISOString(),
        },
      ];
      localStorage.setItem(key, JSON.stringify(initialAddresses));
      return initialAddresses;
    }
    try {
      const list = JSON.parse(stored) as CustomerAddress[];
      // If defaults were provided and list addresses still have generic placeholders, update them live
      if (defaults && (defaults.name || defaults.phone)) {
        let changed = false;
        const updated = list.map((addr) => {
          let modified = false;
          let name = addr.recipient_name;
          let phone = addr.phone;
          if (defaults.name && (!name || name === 'Customer' || name === 'Primary Contact')) {
            name = defaults.name;
            modified = true;
          }
          if (defaults.phone && (!phone || phone === '0803 123 4567')) {
            phone = defaults.phone;
            modified = true;
          }
          if (modified) {
            changed = true;
            return { ...addr, recipient_name: name, phone };
          }
          return addr;
        });
        if (changed) {
          localStorage.setItem(key, JSON.stringify(updated));
          return updated;
        }
      }
      return list;
    } catch {
      return [];
    }
  },

  async getAddress(id: string, userId?: string): Promise<CustomerAddress | null> {
    const list = await this.listAddresses(userId);
    return list.find((a) => a.id === id) || null;
  },

  async getDefaultAddress(userId?: string, defaults?: AddressUserDefaults): Promise<CustomerAddress | null> {
    const list = await this.listAddresses(userId, defaults);
    return list.find((a) => a.is_default) || list[0] || null;
  },

  async createAddress(input: CreateAddressInput, userId?: string): Promise<CustomerAddress> {
    const list = await this.listAddresses(userId);
    const newAddress: CustomerAddress = {
      ...input,
      id: `addr-${Date.now()}-${Math.random().toString(36).substring(2, 7)}`,
      user_id: userId,
      created_at: new Date().toISOString(),
      is_default: input.is_default || list.length === 0,
    };

    let updatedList = list;
    if (newAddress.is_default) {
      updatedList = list.map((a) => ({ ...a, is_default: false }));
    }

    updatedList.unshift(newAddress);
    const key = getStorageKey(userId);
    localStorage.setItem(key, JSON.stringify(updatedList));
    notifyAddressChanged();
    return newAddress;
  },

  async updateAddress(id: string, input: UpdateAddressInput, userId?: string): Promise<CustomerAddress> {
    const list = await this.listAddresses(userId);
    const index = list.findIndex((a) => a.id === id);
    if (index === -1) throw new Error('Address not found');

    let updatedList = list;
    if (input.is_default) {
      updatedList = list.map((a) => ({ ...a, is_default: false }));
    }

    const updatedAddress: CustomerAddress = {
      ...updatedList[index],
      ...input,
    };

    updatedList[index] = updatedAddress;
    const key = getStorageKey(userId);
    localStorage.setItem(key, JSON.stringify(updatedList));
    notifyAddressChanged();
    return updatedAddress;
  },

  async setDefaultAddress(id: string, userId?: string): Promise<CustomerAddress> {
    const res = await this.updateAddress(id, { is_default: true }, userId);
    notifyAddressChanged();
    return res;
  },

  async deleteAddress(id: string, userId?: string): Promise<void> {
    const list = await this.listAddresses(userId);
    const filtered = list.filter((a) => a.id !== id);
    if (filtered.length > 0 && !filtered.some((a) => a.is_default)) {
      filtered[0].is_default = true;
    }
    const key = getStorageKey(userId);
    localStorage.setItem(key, JSON.stringify(filtered));
    notifyAddressChanged();
  },

  async saveLivePrimaryAddress(
    input: {
      recipient_name: string;
      phone: string;
      street: string;
      landmark?: string;
      city: string;
      state: string;
      instructions?: string;
      label?: string;
    },
    userId?: string,
  ): Promise<CustomerAddress> {
    const list = await this.listAddresses(userId);
    const defaultAddr = list.find((a) => a.is_default) || list[0];
    if (defaultAddr) {
      return this.updateAddress(
        defaultAddr.id,
        {
          label: input.label || defaultAddr.label,
          recipient_name: input.recipient_name,
          phone: input.phone,
          street: input.street,
          landmark: input.landmark !== undefined ? input.landmark : defaultAddr.landmark,
          city: input.city,
          state: input.state,
          instructions: input.instructions !== undefined ? input.instructions : defaultAddr.instructions,
          is_default: true,
        },
        userId,
      );
    }
    return this.createAddress(
      {
        label: input.label || 'Main Commercial Store',
        recipient_name: input.recipient_name,
        phone: input.phone,
        street: input.street,
        landmark: input.landmark,
        city: input.city,
        state: input.state,
        instructions: input.instructions,
        is_default: true,
      },
      userId,
    );
  },

  async saveOrUpdateFromCheckout(
    input: {
      recipient_name: string;
      phone: string;
      street: string;
      city: string;
      state: string;
      landmark?: string;
      instructions?: string;
      is_default?: boolean;
    },
    userId?: string,
  ): Promise<CustomerAddress> {
    const list = await this.listAddresses(userId);
    // Look for matching street and city
    const existing = list.find(
      (a) =>
        a.street.trim().toLowerCase() === input.street.trim().toLowerCase() &&
        a.city.trim().toLowerCase() === input.city.trim().toLowerCase(),
    );

    if (existing) {
      return this.updateAddress(
        existing.id,
        {
          recipient_name: input.recipient_name,
          phone: input.phone,
          landmark: input.landmark || existing.landmark,
          instructions: input.instructions || existing.instructions,
          is_default: true,
        },
        userId,
      );
    }

    // Check if the only existing address is the initial default placeholder
    const isSingleDefault = list.length === 1 && list[0].street === '14 Faulks Road';
    if (isSingleDefault) {
      return this.updateAddress(
        list[0].id,
        {
          label: 'Primary Delivery Location',
          recipient_name: input.recipient_name,
          phone: input.phone,
          street: input.street,
          landmark: input.landmark || '',
          city: input.city,
          state: input.state,
          instructions: input.instructions || '',
          is_default: true,
        },
        userId,
      );
    }

    return this.createAddress(
      {
        label: 'Delivery Location',
        recipient_name: input.recipient_name,
        phone: input.phone,
        street: input.street,
        landmark: input.landmark,
        city: input.city,
        state: input.state,
        instructions: input.instructions,
        is_default: true,
      },
      userId,
    );
  },
};
