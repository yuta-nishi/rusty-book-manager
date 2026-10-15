# rusty-book-manager frontend

[SvelteKit 3](https://svelte.dev/docs/kit) (Svelte 5 + TypeScript) の SPA で、
`backend/` の REST API をブラウザから直接呼び出します。

## 前提

- Node.js 24 (`.nvmrc`。SvelteKit 3 の要件は 22.17 以上)
- pnpm 12 (`package.json` の `packageManager` で固定。corepack が自動で用意します)
- 起動中のバックエンド (`mise run back-run`)

バックエンドはフロントエンドの dev サーバからのクロスオリジン呼び出しを想定して
CORS を設定済みで、`/auth/login` が返すアクセストークンを `Authorization: Bearer`
ヘッダに載せる方式です。トークンは `localStorage` に保存します。

## ツールチェーン

| 用途                 | ツール                                                                      |
| -------------------- | --------------------------------------------------------------------------- |
| パッケージマネージャ | pnpm 12                                                                     |
| ランタイム           | Node.js 24 (mise)。mise は Node と pnpm のみ                                |
| ビルド / 開発サーバ  | [Vite+](https://viteplus.dev/) (`vite-plus@1.0.0`) 同梱の Vite 8 + Rolldown |
| テスト               | Vite+ 同梱の Vitest 5                                                       |
| linter               | Oxlint (Vite+ 同梱)                                                         |
| formatter            | Oxfmt (Vite+ 同梱)                                                          |
| 型チェック           | `svelte-check` (`.svelte` の型は tsc 系では見えないため併用)                |

### Vite+ と mise の使い分け

Vite+ にはグローバル CLI (`vp`) もありますが、**これは Node.js とパッケージマネージャを
自身で管理する**ため、mise と競合します。このリポジトリでは

- ランタイムとパッケージマネージャ: mise (+ corepack)
- フロントエンドのツールチェーン: プロジェクトローカルの `vite-plus`

という分担にしています。`vp` は `node_modules/.bin` から解決されるので、`pnpm <script>`
経由で実行します (`pnpm exec vp ...` でも同じ)。グローバル `vp` を後から入れる場合も、
`vp env off` で Node / PM 管理を切れば競合しません。

## セットアップ

```sh
mise install              # Node.js 24 を用意
mise run front-install    # pnpm install
cp frontend/.env.example frontend/.env   # 必要なら PUBLIC_API_BASE_URL を変更
```

## スクリプト

| コマンド                        | 内容                                                               |
| ------------------------------- | ------------------------------------------------------------------ |
| `pnpm dev`                      | dev サーバ (`vp dev`、既定 5173)                                   |
| `pnpm build`                    | プロダクションビルド (`vp build`)。adapter-node で `build/` に出力 |
| `pnpm preview`                  | ビルドをプレビュー (`vp preview`)                                  |
| `pnpm check`                    | フォーマット + lint + 型認識 lint (`vp check`)                     |
| `pnpm check:fix`                | 上記を自動修正 (`vp check --fix`)                                  |
| `pnpm fmt` / `pnpm fmt:check`   | Oxfmt                                                              |
| `pnpm lint`                     | Oxlint                                                             |
| `pnpm typecheck`                | `svelte-check`                                                     |
| `pnpm test` / `pnpm test:watch` | Vitest 5                                                           |

mise 経由のショートカットもあります: `mise run front-install` / `front-dev` /
`front-build` / `front-serve` / `front-fmt` / `front-lint` / `front-typecheck` /
`front-check` / `front-test` / `front-ci`。

ビルドしたものを動かすときは `mise run front-serve` (`front-build` の後に
`PORT=3000 node build`)。ルートの `mise.toml` はバックエンド用に `PORT=8080` を
設定しており、adapter-node も同じ `PORT` を読むため、**明示的に別ポートを指定しないと
バックエンドと衝突します**。

## テスト

`src/**/*.test.ts` を Vitest 5 (jsdom) で実行します。

- `src/lib/api/client.test.ts` — `fetch` ラッパ (URL 組み立て、Bearer ヘッダ、204/空ボディ、エラー)
- `src/lib/api/index.test.ts` — auth が `/auth/*` にマウントされている契約
- `src/lib/session.svelte.test.ts` — トークンの復元/失効/ログイン/ログアウト
- `src/lib/components/BookForm.svelte.test.ts` — 入力、送信、API エラー表示
- `src/lib/format.test.ts` — 日時と ID の表示整形
- `src/routes/(app)/layout.guard.test.ts` — 未ログイン時のリダイレクト
- `src/routes/(app)/books/books.load.test.ts` — ページングパラメータの解釈

`src/vitest-setup.ts` で testing-library の `cleanup` と、`$app/env/public` が参照する
グローバル (`__sveltekit_dev`) を用意しています。後者は SvelteKit の dev サーバが
注入するもので、テストでは存在しないための代替です。

## 環境変数

`PUBLIC_API_BASE_URL` を `src/env.ts` で `defineEnvVars` により宣言しています
(SvelteKit 3 の環境変数機構。`$app/env/public` から import)。既定は
`http://localhost:8080` で、ビルド後も環境変数から解決されます (ビルド時に焼き込まれません)。

## 画面

| パス                   | 内容                                              |
| ---------------------- | ------------------------------------------------- |
| `/login`               | メールアドレスとパスワードでログイン              |
| `/books`               | 書籍一覧 (limit/offset のページング)              |
| `/books/new`           | 書籍の登録                                        |
| `/books/[bookId]`      | 書籍の詳細・貸出/返却・貸出履歴                   |
| `/books/[bookId]/edit` | 書籍の編集 (登録者のみ)                           |
| `/checkouts`           | 貸出中の書籍一覧                                  |
| `/users`               | ユーザー管理 (管理者のみ): 登録・ロール変更・削除 |
| `/me`                  | 現在のユーザー、パスワード変更、自分の貸出中書籍  |

初期データの管理者は `eleazar.fig@example.com` / `password` です
(`backend/data/initial_setup.sql`)。

## 構成

```
src/
  env.ts                     # PUBLIC_API_BASE_URL の宣言
  vitest-setup.ts            # テスト共通の前準備
  app.css                    # グローバルスタイル
  lib/
    api/{client,index,types}.ts   # 型付き API クライアント
    session.svelte.ts             # 認証状態 (runes + localStorage)
    components/BookForm.svelte
    format.ts / test-utils.ts
  routes/
    +layout.ts               # ssr = false (SPA)
    +page.ts                 # / -> /books
    login/
    (app)/                   # 要ログインのレイアウト (+layout.ts でガード)
```

## 実装上の注意

- バックエンドのエラー応答はステータスコードのみでボディが空です。そのため
  `src/lib/api/client.ts` の `messageForStatus` でステータスからメッセージを生成します。
- ID は UUID ですがハイフンなしの小文字 16 進文字列でシリアライズされるため、
  フロントエンドでは不透明な文字列として扱います。
- 書籍の更新・削除は登録者のみ (それ以外は 404)、返却は借りている本人のみ
  (それ以外は 422) という API の仕様に合わせて UI を出し分けています。

## 設定ファイル

- linter / formatter / テスト / Vite の設定はすべて `vite.config.ts` に集約しています
  (Vite+ の推奨。`.oxlintrc.json` や `vitest.config.ts` は使いません)。
- `.svelte` のフォーマットは Oxfmt が同梱の Prettier プラグイン経由で行うため、
  `fmt.svelte: true` と `svelte` パッケージが必要です。
- `vite` は `pnpm-workspace.yaml` の `overrides` で `@voidzero-dev/vite-plus-core` に
  エイリアスしています。Vite+ core は自身のバージョン (1.0.0) を報告するため、
  `vite@^8` を peer に持つ SvelteKit 向けに `peerDependencyRules.allowedVersions` を
  設定しています。

## SvelteKit 3 について

SvelteKit 3 は `svelte.config.js` を廃止し、設定は `vite.config.ts` の
`sveltekit()` プラグインに移動しました。`$lib` は `#lib` (package.json の
`imports`) に、`$app/stores` は `$app/state` に置き換わっています。
