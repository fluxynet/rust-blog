import { useState } from "react";

export type User = {
  name: string;
  avatar_url: string;
  login: string;
};

type AuthContextType = {
  user: User | null;
  logout: () => void;
};

const fakeUser: User = {
  avatar_url: "https://avatars.githubusercontent.com/u/949842?v=4",
  login: "fluxynet",
  name: "Muhammad Yusuf",
};

export const useAuth = (): AuthContextType => {
  const [user, setUser] = useState<User | null>(null);

  const login = (user: User) => {
    setUser(user);
  };

  const logout = () => {
    setUser(null);
  };

  login(fakeUser);

  return { logout, user };
};
