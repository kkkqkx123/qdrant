将一个使用 **Apache License 2.0** 的 Rust 数据库项目改造为嵌入式库（作为 `crate` 内部使用）时，需在以下方面遵守许可证要求并作出声明：

---

### 1. **保留原始版权声明**

- **位置**：在项目的根目录或源代码文件中。
- **内容**：保留原始项目的所有版权声明（通常在 `Cargo.toml`、文件头或 `LICENSE` 文件中）。  
  **示例**（在 `src/lib.rs` 或文件顶部添加）：
  ```rust
  // Copyright [yyyy] [Original Author/Organization]
  // Licensed under the Apache License, Version 2.0 (see LICENSE file).
  ```

---

### 2. **包含完整的许可证文件**

- **位置**：项目的根目录或 `LICENSE` 文件。
- **操作**：
  - 复制原始项目的 `LICENSE` 文件（确保是完整的 Apache 2.0 文本）。
  - 若项目已有 `LICENSE` 文件，需合并内容并明确标注原始许可证的适用部分。

---

### 3. **声明修改内容（如果修改了代码）**

- **位置**：被修改的源文件头部或项目文档（如 `README.md`）。
- **要求**：
  - 在修改过的文件中添加注释，说明“此文件基于 [原始项目名] 修改”。
  - 推荐在 `CHANGELOG.md` 或类似文件中记录重大修改。

  **示例**：

  ```rust
  // This file is modified from [Original Project Name]
  // (https://github.com/xxx/xxx), licensed under Apache 2.0.
  ```

---

### 4. **处理 NOTICE 文件（如果存在）**

- **位置**：根目录的 `NOTICE` 文件。
- **操作**：
  - 如果原始项目有 `NOTICE` 文件（包含第三方声明或专利信息），必须保留其内容。
  - 可追加自己的声明，但不得删除原有内容。

---

### 5. **在 Cargo.toml 中声明许可证**

- **位置**：`Cargo.toml` 文件的 `[package]` 部分。
- **内容**：
  ```toml
  [package]
  name = "your-crate"
  license = "Apache-2.0"
  # 可选：注明原始项目
  description = "Internal embedded database based on [Original Project], Apache 2.0 licensed."
  ```

---

### 6. **文档声明（推荐）**

- **位置**：`README.md` 或项目文档。
- **内容**：  
  示例：
  ```markdown
  ## License

  This project is licensed under the Apache License 2.0.  
  It includes modifications of [Original Project Name], which is also Apache 2.0 licensed.  
  See [LICENSE](LICENSE) and [NOTICE](NOTICE) for details.
  ```

---

### 关键注意事项

1. **无需开源你的项目**：  
   Apache 2.0 允许闭源使用，只要满足上述声明要求。
2. **专利授权**：  
   即使用作内部库，用户仍享有原始项目的专利授权（但需注意诉讼条款）。
3. **依赖传递性**：  
   如果项目依赖其他 Apache 2.0 的库，需同样遵守其许可证要求。

---

### 总结步骤

| 步骤 | 操作                                     |
| ---- | ---------------------------------------- |
| 1    | 保留原始版权声明（代码/文档中）。        |
| 2    | 包含完整的 `LICENSE` 和 `NOTICE` 文件。  |
| 3    | 标注修改过的文件和内容。                 |
| 4    | 在 `Cargo.toml` 和文档中明确许可证信息。 |

完成后，即可合法将改造后的数据库作为内部 `crate` 使用，无需额外授权。

======

对原始代码进行大规模重构（如合并crate）时，需在以下方面进行声明以满足Apache License 2.0的要求：

### 1. **保留原始版权声明**

- 在重构后的代码文件中保留原始版权声明，明确标注原始作者及版权归属。
- **示例**：
  ```rust
  // Copyright [原始年份] [原始作者/组织]
  // Licensed under the Apache License, Version 2.0
  ```

### 2. **标注修改内容**

- 在重构涉及的文件头部添加显著修改声明，说明重构的范围及修改者信息。
- **示例**：
  ```rust
  // Modified by [Your Name/Company] on [Date]
  // Changes: Merged crates A and B into a single embedded library.
  ```

### 3. **包含完整的许可证文件**

- 在项目根目录保留原始`LICENSE`文件（Apache 2.0全文），确保分发时包含该文件。
- 若原始项目包含`NOTICE`文件，需在重构后的项目中保留其内容，可追加新的声明但不得删除原有信息。

### 4. **文档声明**

- 在`README.md`或项目文档中明确说明重构行为，并声明原始项目的许可证信息。
- **示例**：
  ```markdown
  ## License

  This project is based on [Original Project Name], licensed under Apache 2.0.
  Major refactoring includes merging crates for embedded use. See [LICENSE](LICENSE) for details.
  ```

### 5. **Cargo.toml声明**

- 在`Cargo.toml`中注明许可证类型及原始项目信息：
  ```toml
  [package]
  license = "Apache-2.0"
  description = "Refactored embedded database based on [Original Project], Apache 2.0 licensed."
  ```

### 关键注意事项

- **无需开源项目**：Apache 2.0允许闭源使用，即使重构后作为内部crate也无需公开代码。
- **专利授权**：即使用于内部库，用户仍享有原始项目的专利授权（需避免专利诉讼触发终止条款）。
- **兼容性检查**：若重构涉及其他依赖项，需确保其许可证与Apache 2.0兼容（如MIT、BSD）。

### 总结步骤

| 步骤 | 操作                                 |
| ---- | ------------------------------------ |
| 1    | 保留原始版权声明（代码/文档中）。    |
| 2    | 在修改文件中添加重构说明。           |
| 3    | 包含完整的`LICENSE`和`NOTICE`文件。  |
| 4    | 在文档和配置文件中明确标注重构信息。 |

完成上述声明后，即可合法将重构后的数据库作为内部crate使用。
