import { invoke } from "@tauri-apps/api/core";
import { getCurrentRole, getCurrentUser } from "../auth/session";

export type SaleListItem = {
  id: number;
  totalAmount: number;
  note: string | null;
  createdBy: string | null;
  itemCount: number;
  createdAt: string;
};

export type SaleItemInput = {
  productId: number;
  quantity: number;
  unitPrice: number;
};

export type CreateSaleInput = {
  items: SaleItemInput[];
  note?: string;
};

function requireRole(): "admin" | "user" {
  const role = getCurrentRole();
  if (!role) throw new Error("Not authenticated");
  return role;
}

function requireUsername(): string {
  const user = getCurrentUser();
  if (!user) throw new Error("Not authenticated");
  return user.username;
}

export async function fetchSales(): Promise<SaleListItem[]> {
  const requesterRole = requireRole();
  return invoke<SaleListItem[]>("list_sales", { requesterRole });
}

export async function createSale(input: CreateSaleInput) {
  const requesterRole = requireRole();
  const requesterUsername = requireUsername();

  return invoke("create_sale", {
    requesterRole,
    requesterUsername,
    input,
  });
}
