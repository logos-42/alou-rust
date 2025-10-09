-- ============================================
-- Alou Pay - 数据库初始化脚本
-- 版本: 001_init_en
-- ============================================

-- 1. 创建邀请码序列
CREATE SEQUENCE IF NOT EXISTS invitation_codes_id_seq;

-- 2. 创建邀请码表
CREATE TABLE IF NOT EXISTS "public"."invitation_codes" (
  "id" int8 NOT NULL DEFAULT nextval('invitation_codes_id_seq'::regclass),
  "user_id" int8,                                      -- 使用该邀请码的用户ID（激活时绑定）
  "code" varchar(20) COLLATE "pg_catalog"."default" NOT NULL,  -- 邀请码（格式：Alou_XXXXXXXXXX）
  "max_uses" int4 DEFAULT 1,                           -- 最大使用次数（当前未启用此限制）
  "used_count" int4 DEFAULT 0,                         -- 已使用次数
  "status" varchar(20) COLLATE "pg_catalog"."default" DEFAULT 'inactive'::character varying,  -- 状态
  "expires_at" timestamptz(6),                         -- 过期时间（默认30天）
  "created_at" timestamptz(6) DEFAULT now(),           -- 创建时间
  "updated_at" timestamptz(6) DEFAULT now(),           -- 更新时间
  
  CONSTRAINT "invitation_codes_pkey" PRIMARY KEY ("id"),
  CONSTRAINT "invitation_codes_code_key" UNIQUE ("code"),
  CONSTRAINT "invitation_codes_status_check" CHECK (status::text = ANY (ARRAY[
    'active'::character varying, 
    'inactive'::character varying, 
    'expired'::character varying
  ]::text[]))
);

-- 3. 设置表所有者
ALTER TABLE "public"."invitation_codes" 
  OWNER TO "postgres";

-- 4. 创建索引
CREATE INDEX IF NOT EXISTS "idx_invitation_codes_code" 
  ON "public"."invitation_codes" USING btree (
    "code" COLLATE "pg_catalog"."default" "pg_catalog"."text_ops" ASC NULLS LAST
  );

CREATE INDEX IF NOT EXISTS "idx_invitation_codes_user_id" 
  ON "public"."invitation_codes" USING btree (
    "user_id" "pg_catalog"."int8_ops" ASC NULLS LAST
  );

CREATE INDEX IF NOT EXISTS "idx_invitation_codes_status" 
  ON "public"."invitation_codes" USING btree (
    "status" COLLATE "pg_catalog"."default" "pg_catalog"."text_ops" ASC NULLS LAST
  );

CREATE INDEX IF NOT EXISTS "idx_invitation_codes_expires" 
  ON "public"."invitation_codes" USING btree (
    "expires_at" "pg_catalog"."timestamptz_ops" ASC NULLS LAST
  );

-- 5. 添加注释（可选）
COMMENT ON TABLE "public"."invitation_codes" IS '邀请码表 - 管理用户注册邀请码';
COMMENT ON COLUMN "public"."invitation_codes"."code" IS '邀请码，格式：Alou_XXXXXXXXXX';
COMMENT ON COLUMN "public"."invitation_codes"."status" IS '状态：active（已激活）、inactive（未使用）、expired（已过期）';
COMMENT ON COLUMN "public"."invitation_codes"."user_id" IS '使用该邀请码的用户ID（激活时绑定）';