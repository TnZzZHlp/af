import { defineStore } from "pinia";
import { ref, watch } from "vue";
import { login as apiLogin, type LoginResponse } from "@/api/auth";
import { ApiError } from "@/api/client";

const USER_KEY = "af_auth_user";

export const useAuthStore = defineStore("auth", () => {
  const user = ref<LoginResponse | null>((() => {
    const stored = localStorage.getItem(USER_KEY);
    return stored ? JSON.parse(stored) : null;
  })());
  const loading = ref(false);
  const error = ref<string | null>(null);

  watch(user, (val) => {
    if (val) {
      localStorage.setItem(USER_KEY, JSON.stringify(val));
    } else {
      localStorage.removeItem(USER_KEY);
    }
  }, { deep: true });

  async function login(username: string, password: string) {
    loading.value = true;
    error.value = null;
    user.value = null;

    try {
      const response = await apiLogin({ username, password });
      user.value = response;
      return true;
    } catch (e) {
      if (e instanceof ApiError) {
        error.value = e.message;
      } else {
        error.value = "Unable to sign in. Please try again.";
      }
      return false;
    } finally {
      loading.value = false;
    }
  }

  function logout() {
    user.value = null;
    error.value = null;
  }

  return {
    user,
    loading,
    error,
    login,
    logout,
  };
});
