CREATE OR REPLACE FUNCTION decrease_stock_on_sale()
RETURNS TRIGGER AS $$
DECLARE
    current_stock INTEGER;
BEGIN
    SELECT stock_quantity INTO current_stock
    FROM products WHERE id = NEW.product_id FOR UPDATE;

    IF current_stock < NEW.quantity THEN
        RAISE EXCEPTION 'Insufficient stock for product %', NEW.product_id;
    END IF;

    UPDATE products
    SET stock_quantity = stock_quantity - NEW.quantity
    WHERE id = NEW.product_id;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_decrease_stock
    AFTER INSERT ON sale_items
    FOR EACH ROW
    EXECUTE FUNCTION decrease_stock_on_sale();
