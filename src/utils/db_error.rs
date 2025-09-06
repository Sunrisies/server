use sea_orm::DbErr;

/// 把 Sea-ORM 底层数据库错误转成用户能看懂的 &str
pub fn db_err_map(e: DbErr) -> &'static str {
    // Sea-ORM 的 DbErr::Query 里会带数据库原始错误信息
    let detail = e.to_string();

    if detail.contains("duplicate key value violates unique constraint") {
        // 可以细化到字段
        if detail.contains("categories_slug_key") {
            "英文名（slug）已存在，请更换"
        } else if detail.contains("categories_name_key") {
            "分类名称已存在"
        } else {
            "数据重复，请检查唯一字段"
        }
    } else if detail.contains("foreign key constraint") {
        "关联数据不存在，无法操作"
    } else if detail.contains("violates not-null constraint") {
        "必填字段不能为空"
    } else if detail.contains("value too long") {
        "字段长度超出限制"
    } else {
        // 其他数据库错误统一模糊提示
        "数据库操作失败，请稍后再试"
    }
}
