-- TABLE: users
CREATE TABLE users (
  user_id UUID DEFAULT gen_random_uuid () PRIMARY KEY,
  balance NUMERIC(16, 2) NOT NULL DEFAULT 0.00 CHECK (balance >= 0),
  frozen BOOLEAN DEFAULT FALSE NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT timezone ('utc', now ()),
  mc_id UUID UNIQUE,
  disc_id BIGINT UNIQUE CHECK (
    disc_id > 0
    OR disc_id IS NULL
  ),
  CHECK (
    disc_id IS NOT NULL
    OR mc_id IS NOT NULL
  )
);

CREATE UNIQUE INDEX idx_user_mc ON users (mc_id)
WHERE
  mc_id IS NOT NULL;

CREATE UNIQUE INDEX idx_user_disc ON users (disc_id)
WHERE
  disc_id IS NOT NULL;

-- TABLE: stocks
CREATE TABLE stocks (
  ticker VARCHAR(5) PRIMARY KEY CHECK (upper(ticker) = ticker),
  shares INTEGER NOT NULL CHECK (shares > 0),
  created_at TIMESTAMPTZ NOT NULL DEFAULT timezone ('utc', now ()),
  recent_price MONEY DEFAULT 0 NOT NULL,
  frozen BOOLEAN DEFAULT FALSE NOT NULL
);

-- TABLE: holdings
CREATE TABLE holdings (
  ticker VARCHAR(5) NOT NULL,
  user_id UUID NOT NULL,
  shares INTEGER NOT NULL,
  CONSTRAINT shares_nonnegative check (shares >= 0),
  FOREIGN KEY (user_id) REFERENCES users (user_id),
  FOREIGN KEY (ticker) REFERENCES stocks (ticker),
  PRIMARY KEY (user_id, ticker)
);

CREATE INDEX idx_holdings_user ON holdings (user_id);

CREATE INDEX idx_holdings_stock ON holdings (ticker);

CREATE TABLE orders (
  order_id SERIAL PRIMARY KEY,
  user_id UUID NOT NULL,
  ticker VARCHAR(5) NOT NULL,
  price NUMERIC(16, 2) NOT NULL CHECK (price > 0),
  shares INTEGER NOT NULL CHECK (shares > 0),
  -- true: buy, false: sell
  type BOOLEAN NOT NULL,
  FOREIGN KEY (user_id) REFERENCES users (user_id),
  FOREIGN KEY (ticker) REFERENCES stocks (ticker)
);

-- TABLE: stock events
-- Events pertaining to stocks being bought or sold
CREATE TABLE stock_events (
  event_id SERIAL PRIMARY KEY,
  time TIMESTAMPTZ NOT NULL DEFAULT timezone ('utc', now ()),
  seller_id UUID NOT NULL,
  buyer_id UUID NOT NULL,
  ticker VARCHAR(5) NOT NULL,
  price NUMERIC(16, 2) NOT NULL CHECK (price > 0),
  shares INTEGER NOT NULL CHECK (shares > 0),
  FOREIGN KEY (seller_id) REFERENCES users (user_id),
  FOREIGN KEY (buyer_id) REFERENCES users (user_id),
  FOREIGN KEY (ticker) REFERENCES stocks (ticker)
);
