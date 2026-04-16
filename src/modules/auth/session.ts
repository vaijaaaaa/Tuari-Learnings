import { invoke } from "@tauri-apps/api/core";

export type Role = "admin" | "user"

export type AuthUser = {
    id : number;
    fullName : string;
    username : string;
    role : Role;
};

const SESSION_KEY = "framstack.auth.user";

function isRole(value:string):value is Role {
    return value === "admin" || value === "user";
}

export async function login(username:string,password:string): Promise<AuthUser>{
    const user = await invoke<AuthUser>("login",{username,password});
    localStorage.setItem(SESSION_KEY,JSON.stringify(user));
    return user;
}

export function getCurrentUser() : AuthUser | null{
    const raw = localStorage.getItem(SESSION_KEY);
    if(!raw) return null;

    try {
        const parsed = JSON.parse(raw) as AuthUser;
        if(!parsed || !isRole(parsed.role)){
            localStorage.removeItem(SESSION_KEY);
            return null;
        }
        return parsed;
    } catch (error) {
        localStorage.removeItem(SESSION_KEY);
        return null;
    }
}

export function getCurrentRole():Role | null{
    return getCurrentUser()?.role ?? null;
}

export function logout():void {
    localStorage.removeItem(SESSION_KEY);
}