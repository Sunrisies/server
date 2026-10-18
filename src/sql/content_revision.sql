CREATE OR REPLACE FUNCTION bump_public_content_revision() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
    UPDATE public_content_revision SET version=version+1 WHERE id=1;
    RETURN NULL;
END;
$$;
DROP TRIGGER IF EXISTS content_insert_delete ON posts;
CREATE TRIGGER content_insert_delete AFTER INSERT OR DELETE ON posts FOR EACH STATEMENT EXECUTE FUNCTION bump_public_content_revision();
DROP TRIGGER IF EXISTS content_edit ON posts;
CREATE TRIGGER content_edit AFTER UPDATE OF title,summary,content,markdowncontent,cover_image,category_id,status,featured,published_at,updated_at ON posts FOR EACH STATEMENT EXECUTE FUNCTION bump_public_content_revision();
DROP TRIGGER IF EXISTS content_change ON categories;
CREATE TRIGGER content_change AFTER INSERT OR UPDATE OR DELETE ON categories FOR EACH STATEMENT EXECUTE FUNCTION bump_public_content_revision();
DROP TRIGGER IF EXISTS content_change ON tags;
CREATE TRIGGER content_change AFTER INSERT OR UPDATE OR DELETE ON tags FOR EACH STATEMENT EXECUTE FUNCTION bump_public_content_revision();
DROP TRIGGER IF EXISTS content_change ON post_tags;
CREATE TRIGGER content_change AFTER INSERT OR UPDATE OR DELETE ON post_tags FOR EACH STATEMENT EXECUTE FUNCTION bump_public_content_revision();
DROP TRIGGER IF EXISTS content_change ON external_links;
CREATE TRIGGER content_change AFTER INSERT OR UPDATE OR DELETE ON external_links FOR EACH STATEMENT EXECUTE FUNCTION bump_public_content_revision();

DROP TRIGGER IF EXISTS author_profile_change ON users;
CREATE TRIGGER author_profile_change AFTER UPDATE OF image,user_name ON users FOR EACH STATEMENT EXECUTE FUNCTION bump_public_content_revision();
