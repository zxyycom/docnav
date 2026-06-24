# Web 接口模式（Web Interface Patterns）

只有当用户明确要求通用 REST、GraphQL 或 TypeScript interface design 时，才读取本 reference。处理 Docnav protocol、adapter、CLI、schema contract 时，使用 `../SKILL.md` 并按需读取 `docnav-contract-scope.md`。

## REST 模式（Patterns）

- Endpoint 使用复数 resource noun。
- Error response 保持一种 structured shape。
- List endpoint 从一开始就设计 pagination。
- Filtering 和 sorting 使用 query parameter。
- 稀疏变更优先使用 partial update。

示例：

```text
GET    /api/tasks
POST   /api/tasks
GET    /api/tasks/:id
PATCH  /api/tasks/:id
DELETE /api/tasks/:id
```

## TypeScript 模式（Patterns）

使用明确的 input/output type：

```typescript
interface CreateTaskInput {
  title: string;
  description?: string;
}

interface Task {
  id: string;
  title: string;
  description: string | null;
  createdAt: Date;
  updatedAt: Date;
}
```

表达 variant 时，优先使用 discriminated union：

```typescript
type TaskStatus =
  | { type: "pending" }
  | { type: "in_progress"; assignee: string }
  | { type: "completed"; completedAt: Date };
```

这些 pattern 只是可选背景材料，不是 Docnav skill 的默认触发内容。
