-- Function to update collection bookmark count
CREATE OR REPLACE FUNCTION update_collection_bookmark_count()
RETURNS TRIGGER AS $$
BEGIN
    IF TG_OP = 'INSERT' THEN
        UPDATE collections 
        SET bookmark_count = bookmark_count + 1 
        WHERE id = NEW.collection_id;
        RETURN NEW;
    ELSIF TG_OP = 'UPDATE' THEN
        IF OLD.collection_id IS DISTINCT FROM NEW.collection_id THEN
            IF OLD.collection_id IS NOT NULL THEN
                UPDATE collections 
                SET bookmark_count = bookmark_count - 1 
                WHERE id = OLD.collection_id;
            END IF;
            IF NEW.collection_id IS NOT NULL THEN
                UPDATE collections 
                SET bookmark_count = bookmark_count + 1 
                WHERE id = NEW.collection_id;
            END IF;
        END IF;
        RETURN NEW;
    ELSIF TG_OP = 'DELETE' THEN
        IF OLD.collection_id IS NOT NULL THEN
            UPDATE collections 
            SET bookmark_count = bookmark_count - 1 
            WHERE id = OLD.collection_id;
        END IF;
        RETURN OLD;
    END IF;
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

-- Create trigger for collection bookmark count
CREATE TRIGGER update_collection_bookmark_count_trigger
    AFTER INSERT OR UPDATE OR DELETE ON bookmarks
    FOR EACH ROW EXECUTE FUNCTION update_collection_bookmark_count();

-- Function to update tag usage count
CREATE OR REPLACE FUNCTION update_tag_usage_count()
RETURNS TRIGGER AS $$
BEGIN
    IF TG_OP = 'INSERT' THEN
        UPDATE tags 
        SET usage_count = usage_count + 1 
        WHERE id = NEW.tag_id;
        RETURN NEW;
    ELSIF TG_OP = 'DELETE' THEN
        UPDATE tags 
        SET usage_count = usage_count - 1 
        WHERE id = OLD.tag_id;
        RETURN OLD;
    END IF;
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

-- Create trigger for tag usage count
CREATE TRIGGER update_tag_usage_count_trigger
    AFTER INSERT OR DELETE ON bookmark_tags
    FOR EACH ROW EXECUTE FUNCTION update_tag_usage_count();