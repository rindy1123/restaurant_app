CREATE TABLE IF NOT EXISTS table_order_items (
  id SERIAL PRIMARY KEY,
  table_id INT NOT NULL,
  menu_item_id INT NOT NULL,
  prep_time_minutes INT NOT NULL,
  created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
  CONSTRAINT fk_table_id
      FOREIGN KEY(table_id)
      REFERENCES tables(id)
      ON DELETE CASCADE,
  CONSTRAINT fk_menu_item_id
      FOREIGN KEY(menu_item_id)
      REFERENCES menu_items(id)
      ON DELETE CASCADE
);
CREATE INDEX idx_table_id ON table_order_items(table_id);
