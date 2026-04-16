import { invoke } from "@tauri-apps/api/core";
import { getCurrentRole } from "../../auth/session";

export type UserListItem = {
  id: number;
  fullName: string;
  username: string;
  role: string;
  isActive: boolean;
  createdAt: string;
};

export type CreateUserInput = {
  fullName: string;
  username: string;
  password: string;
  role: "admin" | "user";
};

function requireRole(): "admin" | "user" {
  const role = getCurrentRole();
  if (!role) {
    throw new Error("Not authenticated");
  }
  return role;
}

export async function fetchUsers(): Promise<UserListItem[]> {
  const requesterRole = requireRole();

  const users = await invoke<UserListItem[]>("list_users", {
    requesterRole,
  });

  return users;
}

export async function createUser(input: CreateUserInput): Promise<UserListItem> {
  const requesterRole = requireRole();

  const created = await invoke<UserListItem>("create_user", {
    requesterRole,
    input,
  });

  return created;
}