import {invoke} from "@tauri-apps/api/core";
import { getCurrentRole } from "../../auth/session";

export type UserListItem = {
    id : number;
    fullName : string;
    username : string;
    role : string;
    isActive : boolean;
    createdAt:string;
}

export async function fetchUsers():Promise<UserListItem[]>{
    const role = getCurrentRole();

    if(!role){
        throw new Error("Not authenticated");
    }

    const users = await invoke<UserListItem[]>("list_users",{
        requestRole : role,
    });

    return users;
}