# 语言示例

## 目录

- [TypeScript / JavaScript](#typescript--javascript)
- [Python](#python)
- [React / JSX](#react--jsx)

## TypeScript / JavaScript

删除只转发 Promise 的无意义 `async` / `await`：

```typescript
// Before
async function getUser(id: string): Promise<User> {
  return await userService.findById(id);
}

// After
function getUser(id: string): Promise<User> {
  return userService.findById(id);
}
```

简化直接条件赋值，但确认 falsy 语义正确：

```typescript
// Before
let displayName: string;
if (user.nickname) {
  displayName = user.nickname;
} else {
  displayName = user.fullName;
}

// After
const displayName = user.nickname || user.fullName;
```

直接过滤列表时可用数组 API：

```typescript
// Before
const activeUsers: User[] = [];
for (const user of users) {
  if (user.isActive) {
    activeUsers.push(user);
  }
}

// After
const activeUsers = users.filter((user) => user.isActive);
```

删除冗余 boolean 分支：

```typescript
// Before
function isValid(input: string): boolean {
  if (input.length > 0 && input.length < 100) {
    return true;
  }
  return false;
}

// After
function isValid(input: string): boolean {
  return input.length > 0 && input.length < 100;
}
```

## Python

简单映射可用 comprehension：

```python
# Before
result = {}
for item in items:
    result[item.id] = item.name

# After
result = {item.id: item.name for item in items}
```

深层校验优先 guard clause，并保持异常类型和顺序：

```python
# Before
def process(data):
    if data is not None:
        if data.is_valid():
            if data.has_permission():
                return do_work(data)
            else:
                raise PermissionError("No permission")
        else:
            raise ValueError("Invalid data")
    else:
        raise TypeError("Data is None")

# After
def process(data):
    if data is None:
        raise TypeError("Data is None")
    if not data.is_valid():
        raise ValueError("Invalid data")
    if not data.has_permission():
        raise PermissionError("No permission")
    return do_work(data)
```

## React / JSX

重复 render 分支可以合并，但保持 props 和语义一致：

```tsx
// Before
function UserBadge({ user }: Props) {
  if (user.isAdmin) {
    return <Badge variant="admin">Admin</Badge>;
  } else {
    return <Badge variant="default">User</Badge>;
  }
}

// After
function UserBadge({ user }: Props) {
  const variant = user.isAdmin ? "admin" : "default";
  const label = user.isAdmin ? "Admin" : "User";
  return <Badge variant={variant}>{label}</Badge>;
}
```

prop drilling 可能适合用 composition 或 context 处理，但这是设计判断。标记风险并检查现有架构，不要自动大范围改写组件树。
