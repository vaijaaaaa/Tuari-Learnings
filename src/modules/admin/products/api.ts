import { invoke } from "@tauri-apps/api/core";
import { getCurrentRole } from "../../auth/session";

export type ProductListItem = {
  id: number;
  sku: string;
  name: string;
  description: string | null;
  seasonId: number | null;
  seasonName: string | null;
  isActive: boolean;
  createdAt: string;
};

export type CreateProductInput = {
  sku: string;
  name: string;
  description?: string;
  seasonId?: number;
};

function requireRole(): "admin" | "user" {
  const role = getCurrentRole();
  if (!role) {
    throw new Error("Not authenticated");
  }
  return role;
}

export async function fetchProducts(): Promise<ProductListItem[]> {
  const requesterRole = requireRole();

  const products = await invoke<ProductListItem[]>("list_products", {
    requesterRole,
  });

  return products;
}

export async function createProduct(input: CreateProductInput): Promise<ProductListItem> {
  const requesterRole = requireRole();

  const created = await invoke<ProductListItem>("create_product", {
    requesterRole,
    input,
  });

  return created;
}
