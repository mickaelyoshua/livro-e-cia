use std::fmt::Display;

use clap::Parser;
use validator::Validate;

#[derive(Debug, Parser)]
#[command(name = "create-admin")]
#[command(about = "Create an admin user account", long_about = None)]
struct Args {
    #[arg(short, long)]
    email: String,

    #[arg(short, long)]
    name: String,
}

#[derive(Debug)]
enum CliError {
    InvalidEmail(String),
    DatabaseError(String),
    EmailAlreadyExists(String),
    RoleNotFound,
    PasswordTooShort,
}

impl Display for CliError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CliError::InvalidEmail(email) => write!(f, "Invalid email format: {}", email),
            CliError::DatabaseError(msg) => write!(f, "Database error: {}", msg),
            CliError::EmailAlreadyExists(email) => write!(f, "Email {} is already in use", email),
            CliError::RoleNotFound => write!(f, "Admin role not found in database"),
            CliError::PasswordTooShort => write!(f, "Password must be at least 8 characters"),
        }
    }
}

impl std::error::Error for CliError {}

#[derive(Debug, Validate)]
struct EmailValidator {
    #[validate(email, length(max = 255))]
    email: String,
}

fn validate_email(email: &str) -> Result<(), CliError> {
    let validator = EmailValidator {
        email: email.to_string(),
    };

    validator
        .validate()
        .map_err(|e| CliError::InvalidEmail(format!("{}", e)))?;

    Ok(())
}

fn get_password() -> Result<String, CliError> {
    println!("\nEnter password for admin (min 8 characters):");
    let password = rpassword::read_password()
        .map_err(|e| CliError::DatabaseError(format!("Failed to read password: {}", e)))?;

    if password.len() < 8 {
        return Err(CliError::PasswordTooShort);
    }

    println!("Confirm password:");
    let password_confirm = rpassword::read_password()
        .map_err(|e| CliError::DatabaseError(format!("Failed to read password: {}", e)))?;

    if password != password_confirm {
        return Err(CliError::DatabaseError(
            "Passwords do not match".to_string(),
        ));
    }

    Ok(password)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    use backend::{
        auth::password::hash_password,
        models::{NewUser, User},
        schema::{roles, users},
    };
    use diesel::prelude::*;
    use dotenv::dotenv;
    use std::env;
    use uuid::Uuid;

    dotenv().ok();

    let args = Args::parse();

    println!("=== Create Admin Account ===\n");
    println!("Email: {}", args.email);
    println!("Name: {}", args.name);

    validate_email(&args.email)?;

    let password = get_password()?;

    let database_url = env::var("DATABASE_URL")
        .map_err(|_| CliError::DatabaseError("DATABASE_URL not set".to_string()))?;

    println!("\nConnecting to database...");
    let mut conn = PgConnection::establish(&database_url)
        .map_err(|e| CliError::DatabaseError(format!("Failed to connect: {}", e)))?;

    println!("Looking up admin role...");
    let admin_role = roles::table
        .filter(roles::name.eq("admin"))
        .select(roles::id)
        .first::<Uuid>(&mut conn)
        .optional()
        .map_err(|e| CliError::DatabaseError(format!("Failed to query roles: {}", e)))?
        .ok_or(CliError::RoleNotFound)?;

    println!("Checking if email is available...");
    let existing_user = users::table
        .filter(users::email.eq(&args.email))
        .select(users::id)
        .first::<Uuid>(&mut conn)
        .optional()
        .map_err(|e| CliError::DatabaseError(format!("Failed to check email: {}", e)))?;

    if existing_user.is_some() {
        return Err(Box::new(CliError::EmailAlreadyExists(args.email.clone())));
    }

    println!("Hashing password (this may take a moment)...");
    let password_hash = hash_password(&password)
        .map_err(|e| CliError::DatabaseError(format!("Failed to hash password: {}", e)))?;

    println!("Creating admin account...");
    let new_admin = NewUser {
        email: args.email.clone(),
        password_hash,
        name: args.name.clone(),
        role_id: admin_role,
    };

    let inserted_user: User = diesel::insert_into(users::table)
        .values(&new_admin)
        .returning(User::as_returning())
        .get_result(&mut conn)
        .map_err(|e| CliError::DatabaseError(format!("Failed to insert user: {}", e)))?;

    // Mark admin as verified (no email verification needed)
    use chrono::Utc;
    diesel::update(users::table.filter(users::id.eq(inserted_user.id)))
        .set((
            users::email_verified.eq(true),
            users::email_verified_at.eq(Some(Utc::now())),
        ))
        .execute(&mut conn)
        .map_err(|e| CliError::DatabaseError(format!("Failed to verify email: {}", e)))?;

    println!("\n✅ Admin account created successfully!");
    println!("   Email: {}", inserted_user.email);
    println!("   ID: {}", inserted_user.id);
    println!("\nYou can now login with these credentials.");

    Ok(())
}
