import { useState, useEffect } from "react";
import {getBlog} from "@/api/blog";

const { me } = getBlog();

export type User = {
  name: string;
  avatar_url: string;
  login: string;
};

type AuthContextType = {
  user: User | null;
  isLoggedIn: boolean;
};

export const useAuth = (): AuthContextType => {
  const [user, setUser] = useState<User | null>(null);
  const [isLoggedIn, setIsLoggedIn] = useState(false);

  const login = () => {
    if (isLoggedIn) return;
    me().then(({data}) => {
      const { name, avatar_url, login } = data;
      setUser({name, avatar_url, login});
      setIsLoggedIn(true);
    }).catch(() => {
      setUser(null);
      setIsLoggedIn(false);
    });
  };

  useEffect(() => {
    login();
  }, []);

  return { user, isLoggedIn };
};
