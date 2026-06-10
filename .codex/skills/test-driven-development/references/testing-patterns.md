# 测试模式参考

这是跨技术栈通用 testing patterns 的速查参考。与 `test-driven-development` skill 一起使用。

## 目录

- [测试结构（Arrange-Act-Assert）](#测试结构arrange-act-assert)
- [测试命名约定](#测试命名约定)
- [常见 Assertions](#常见-assertions)
- [写好测试](#写好测试)
- [Mock 模式（Mocking Patterns）](#mock-模式mocking-patterns)
- [组件测试（React/Component Testing）](#组件测试reactcomponent-testing)
- [集成测试（API / Integration Testing）](#集成测试api--integration-testing)
- [E2E 测试（Playwright）](#e2e-测试playwright)
- [测试反模式（Anti-Patterns）](#测试反模式anti-patterns)

## 测试结构（Arrange-Act-Assert）

```typescript
it('describes expected behavior', () => {
  // Arrange: 准备测试数据和前置条件
  const input = { title: 'Test Task', priority: 'high' };

  // Act: 执行被测试动作
  const result = createTask(input);

  // Assert: 验证结果
  expect(result.title).toBe('Test Task');
  expect(result.priority).toBe('high');
  expect(result.status).toBe('pending');
});
```

## 测试命名约定

```typescript
// Pattern: [unit] [expected behavior] [condition]
describe('TaskService.createTask', () => {
  it('creates a task with default pending status', () => {});
  it('throws ValidationError when title is empty', () => {});
  it('trims whitespace from title', () => {});
  it('generates a unique ID for each task', () => {});
});
```

## 常见 Assertions

```typescript
// Equality
expect(result).toBe(expected);           // 严格相等 (===)
expect(result).toEqual(expected);        // 深度相等 (objects/arrays)
expect(result).toStrictEqual(expected);  // 深度相等 + 类型匹配

// Truthiness
expect(result).toBeTruthy();
expect(result).toBeFalsy();
expect(result).toBeNull();
expect(result).toBeDefined();
expect(result).toBeUndefined();

// Numbers
expect(result).toBeGreaterThan(5);
expect(result).toBeLessThanOrEqual(10);
expect(result).toBeCloseTo(0.3, 5);      // 浮点数

// Strings
expect(result).toMatch(/pattern/);
expect(result).toContain('substring');

// Arrays / Objects
expect(array).toContain(item);
expect(array).toHaveLength(3);
expect(object).toHaveProperty('key', 'value');

// Errors
expect(() => fn()).toThrow();
expect(() => fn()).toThrow(ValidationError);
expect(() => fn()).toThrow('specific message');

// Async
await expect(asyncFn()).resolves.toBe(value);
await expect(asyncFn()).rejects.toThrow(Error);
```

## 写好测试

- 测试 observable state 和 outputs，不测试内部 call sequences。
- 优先写 DAMP tests：每个测试都应像一小段行为规格，即使这会重复一些 setup。
- 优先使用 real implementations，其次 fakes，再其次 stubs。只有在真实依赖 slow、non-deterministic 或 unsafe 的边界上，才谨慎使用 mocks。
- 每个测试聚焦一个 behavior concept。多个 assertions 可以存在，只要它们证明的是同一概念。
- 用 expected behavior 和 condition 命名测试，不用 implementation detail 命名。
- 保持测试 deterministic：隔离状态、控制时间、避免顺序依赖，并始终 `await` async work。

## Mock 模式（Mocking Patterns）

### 函数 Mock（Mock Functions）

```typescript
const mockFn = jest.fn();
mockFn.mockReturnValue(42);
mockFn.mockResolvedValue({ data: 'test' });
mockFn.mockImplementation((x) => x * 2);

expect(mockFn).toHaveBeenCalled();
expect(mockFn).toHaveBeenCalledWith('arg1', 'arg2');
expect(mockFn).toHaveBeenCalledTimes(3);
```

### 模块 Mock（Mock Modules）

```typescript
// Mock 整个 module
jest.mock('./database', () => ({
  query: jest.fn().mockResolvedValue([{ id: 1, title: 'Test' }]),
}));

// Mock 指定 exports
jest.mock('./utils', () => ({
  ...jest.requireActual('./utils'),
  generateId: jest.fn().mockReturnValue('test-id'),
}));
```

### 只在边界 Mock（Mock at Boundaries Only）

```
可以 mock:                      不要 mock:
├── Database calls              ├── Internal utility functions
├── HTTP requests               ├── Business logic
├── File system operations      ├── Data transformations
├── External API calls          ├── Validation functions
└── Time/Date (when needed)     └── Pure functions
```

## 组件测试（React/Component Testing）

```tsx
import { render, screen, fireEvent, waitFor } from '@testing-library/react';

describe('TaskForm', () => {
  it('submits the form with entered data', async () => {
    const onSubmit = jest.fn();
    render(<TaskForm onSubmit={onSubmit} />);

    // 用 accessible role/label 找元素，不用 test IDs
    await screen.findByRole('textbox', { name: /title/i });
    fireEvent.change(screen.getByRole('textbox', { name: /title/i }), {
      target: { value: 'New Task' },
    });
    fireEvent.click(screen.getByRole('button', { name: /create/i }));

    await waitFor(() => {
      expect(onSubmit).toHaveBeenCalledWith({ title: 'New Task' });
    });
  });

  it('shows validation error for empty title', async () => {
    render(<TaskForm onSubmit={jest.fn()} />);

    fireEvent.click(screen.getByRole('button', { name: /create/i }));

    expect(await screen.findByText(/title is required/i)).toBeInTheDocument();
  });
});
```

## 集成测试（API / Integration Testing）

```typescript
import request from 'supertest';
import { app } from '../src/app';

describe('POST /api/tasks', () => {
  it('creates a task and returns 201', async () => {
    const response = await request(app)
      .post('/api/tasks')
      .send({ title: 'Test Task' })
      .set('Authorization', `Bearer ${testToken}`)
      .expect(201);

    expect(response.body).toMatchObject({
      id: expect.any(String),
      title: 'Test Task',
      status: 'pending',
    });
  });

  it('returns 422 for invalid input', async () => {
    const response = await request(app)
      .post('/api/tasks')
      .send({ title: '' })
      .set('Authorization', `Bearer ${testToken}`)
      .expect(422);

    expect(response.body.error.code).toBe('VALIDATION_ERROR');
  });

  it('returns 401 without authentication', async () => {
    await request(app)
      .post('/api/tasks')
      .send({ title: 'Test' })
      .expect(401);
  });
});
```

## E2E 测试（Playwright）

```typescript
import { test, expect } from '@playwright/test';

test('user can create and complete a task', async ({ page }) => {
  // 导航并认证
  await page.goto('/');
  await page.fill('[name="email"]', 'test@example.com');
  await page.fill('[name="password"]', 'testpass123');
  await page.click('button:has-text("Log in")');

  // 创建 task
  await page.click('button:has-text("New Task")');
  await page.fill('[name="title"]', 'Buy groceries');
  await page.click('button:has-text("Create")');

  // 验证 task 出现
  await expect(page.locator('text=Buy groceries')).toBeVisible();

  // 完成 task
  await page.click('[aria-label="Complete Buy groceries"]');
  await expect(page.locator('text=Buy groceries')).toHaveCSS(
    'text-decoration-line', 'line-through'
  );
});
```

## 测试反模式（Anti-Patterns）

| Anti-Pattern | 问题 | 更好的做法 |
|---|---|---|
| Testing implementation details | refactor 时容易破 | Test inputs/outputs |
| Snapshot everything | 没人认真 review snapshot diffs | Assert specific values |
| Shared mutable state | 测试互相污染 | 每个 test 单独 setup/teardown |
| Testing third-party code | 浪费时间，且不是你的 bug | Mock the boundary |
| Skipping tests to pass CI | 隐藏真实问题 | 修复或删除该 test |
| Permanently using `test.skip` | 形成 dead code | 删除或修复它 |
| Overly broad assertions | 抓不住 regression | 具体断言 |
| No async error handling | 吞掉错误，造成 false passes | 始终 `await` async tests |
