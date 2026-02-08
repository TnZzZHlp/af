import type { ClassValue } from "clsx"
import { clsx } from "clsx"
import { twMerge } from "tailwind-merge"
import Prism from "prismjs";
import "prismjs/components/prism-json";

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs))
}

export function decodeBody(body: number[] | null): string {
  if (!body) return "Empty";
  try {
    const text = new TextDecoder().decode(new Uint8Array(body));
    try {
      // Try to format JSON
      return JSON.stringify(JSON.parse(text), null, 2);
    } catch {
      // Attempt to parse SSE
      if (text.includes('data:')) {
        const lines = text.split('\n');
        const processed = lines.map(line => {
          const trimmed = line.trim();
          if (trimmed.startsWith('data:')) {
            const content = trimmed.substring(5).trim();
            if (content === '[DONE]') return line;
            try {
              const parsed = JSON.parse(content);
              return `data: ${JSON.stringify(parsed, null, 2)}`;
            } catch {
              return line;
            }
          }
          return line;
        });
        return processed.join('\n');
      }
      return text;
    }
  } catch {
    return `[Binary Data: ${body.length} bytes]`;
  }
}

export function getHighlightedHtml(body: number[] | null): string {
  const text = decodeBody(body);
  const trimmed = text.trim();
  if (trimmed.startsWith("{") || trimmed.startsWith("[")) {
    try {
      return Prism.highlight(text, Prism.languages.json as Prism.Grammar, "json");
    } catch (e) {
      console.warn("Prism highlight failed", e);
    }
  }
  // Escape HTML characters for safety since we are using v-html
  return text.replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;")
    .replace(/'/g, "&#039;");
}

interface OpenAiChoice {
  message?: { content?: string };
  delta?: { content?: string };
}

interface OpenAiResponse {
  choices?: OpenAiChoice[];
}

interface AnthropicBlock {
  type: string;
  text?: string;
}

interface AnthropicResponse {
  content?: AnthropicBlock[];
  type?: string;
  delta?: { text?: string };
}

export function extractAiContent(body: number[] | null, apiType: string | null): string | null {
  if (!body || !apiType) return null;

  try {
    const text = new TextDecoder().decode(new Uint8Array(body));

    // We try to parse as JSON first (Non-streaming)
    try {
      const json = JSON.parse(text) as unknown;
      if (apiType === 'openai_chat_completions' || apiType === 'openai_responses') {
        const openAiJson = json as OpenAiResponse;
        if (openAiJson.choices?.[0]?.message?.content) {
          return openAiJson.choices[0].message.content;
        }
      } else if (apiType === 'anthropic_messages') {
        const anthropicJson = json as AnthropicResponse;
        if (Array.isArray(anthropicJson.content)) {
          return anthropicJson.content
            .filter((block) => block.type === 'text')
            .map((block) => block.text || '')
            .join('');
        }
      }
    } catch {
      // JSON parse failed, try SSE
    }

    // Attempt SSE parsing
    if (text.includes('data:')) {
      const lines = text.split('\n');
      let content = '';
      for (const line of lines) {
        const trimmed = line.trim();
        if (trimmed.startsWith('data:')) {
          const dataPart = trimmed.substring(5).trim();
          if (dataPart === '[DONE]') continue;
          try {
            const json = JSON.parse(dataPart) as unknown;
            if (apiType === 'openai_chat_completions' || apiType === 'openai_responses') {
              const openAiJson = json as OpenAiResponse;
              if (openAiJson.choices?.[0]?.delta?.content) {
                content += openAiJson.choices[0].delta.content;
              }
            } else if (apiType === 'anthropic_messages') {
              const anthropicJson = json as AnthropicResponse;
              if (anthropicJson.type === 'content_block_delta' && anthropicJson.delta?.text) {
                content += anthropicJson.delta.text;
              }
            }
          } catch {
            // Ignore parse errors for individual chunks
          }
        }
      }
      return content || null;
    }
  } catch {
    return null;
  }
  return null;
}
