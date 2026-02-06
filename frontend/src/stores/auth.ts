import { defineStore } from "pinia";
import { ref } from "vue";
import { login as apiLogin, type LoginResponse } from "@/api/auth";
import { ApiError } from "@/api/client";

export const useAuthStore = defineStore("auth", () => {
  const user = ref<LoginResponse | null>(null);
  const loading = ref(false);
  const error = ref<string | null>(null);

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
