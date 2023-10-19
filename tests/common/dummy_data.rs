#![allow(dead_code)]
use actix_web::test::{self, TestRequest};
use labrinth::{
    models::projects::Project,
    models::{organizations::Organization, pats::Scopes, projects::Version},
};
use serde_json::json;
use sqlx::Executor;

use crate::common::{actix::AppendsMultipart, database::USER_USER_PAT};

use super::{
    actix::{MultipartSegment, MultipartSegmentData},
    environment::TestEnvironment,
    request_data::get_public_project_creation_data,
};

pub const DUMMY_DATA_UPDATE: i64 = 1;

#[allow(dead_code)]
pub const DUMMY_CATEGORIES: &[&str] = &[
    "combat",
    "decoration",
    "economy",
    "food",
    "magic",
    "mobs",
    "optimization",
];

#[allow(dead_code)]
pub enum DummyJarFile {
    DummyProjectAlpha,
    DummyProjectBeta,
    BasicMod,
    BasicModDifferent,
}

#[allow(dead_code)]
pub enum DummyImage {
    SmallIcon, // 200x200
}

#[derive(Clone)]
pub struct DummyData {
    pub project_alpha: DummyProjectAlpha,
    pub project_beta: DummyProjectBeta,
    pub organization_zeta: DummyOrganizationZeta,
}

#[derive(Clone)]
pub struct DummyProjectAlpha {
    // Alpha project:
    // This is a dummy project created by USER user.
    // It's approved, listed, and visible to the public.
    pub project_id: String,
    pub project_slug: String,
    pub version_id: String,
    pub thread_id: String,
    pub file_hash: String,
    pub team_id: String,
}

#[derive(Clone)]
pub struct DummyProjectBeta {
    // Beta project:
    // This is a dummy project created by USER user.
    // It's not approved, unlisted, and not visible to the public.
    pub project_id: String,
    pub project_slug: String,
    pub version_id: String,
    pub thread_id: String,
    pub file_hash: String,
    pub team_id: String,
}

#[derive(Clone)]
pub struct DummyOrganizationZeta {
    // Zeta organization:
    // This is a dummy organization created by USER user.
    // There are no projects in it.
    pub organization_id: String,
    pub organization_title: String,
    pub team_id: String,
}

pub async fn add_dummy_data(test_env: &TestEnvironment) -> DummyData {
    // Adds basic dummy data to the database directly with sql (user, pats)
    let pool = &test_env.db.pool.clone();

    pool.execute(
        include_str!("../files/dummy_data.sql")
            .replace("$1", &Scopes::all().bits().to_string())
            .as_str(),
    )
    .await
    .unwrap();

    let (alpha_project, alpha_version) = add_project_alpha(test_env).await;
    let (beta_project, beta_version) = add_project_beta(test_env).await;

    let zeta_organization = add_organization_zeta(test_env).await;

    sqlx::query("INSERT INTO dummy_data (update_id) VALUES ($1)")
        .bind(DUMMY_DATA_UPDATE)
        .execute(pool)
        .await
        .unwrap();

    DummyData {
        project_alpha: DummyProjectAlpha {
            team_id: alpha_project.team.to_string(),
            project_id: alpha_project.id.to_string(),
            project_slug: alpha_project.slug.unwrap(),
            version_id: alpha_version.id.to_string(),
            thread_id: alpha_project.thread_id.to_string(),
            file_hash: alpha_version.files[0].hashes["sha1"].clone(),
        },

        project_beta: DummyProjectBeta {
            team_id: beta_project.team.to_string(),
            project_id: beta_project.id.to_string(),
            project_slug: beta_project.slug.unwrap(),
            version_id: beta_version.id.to_string(),
            thread_id: beta_project.thread_id.to_string(),
            file_hash: beta_version.files[0].hashes["sha1"].clone(),
        },

        organization_zeta: DummyOrganizationZeta {
            organization_id: zeta_organization.id.to_string(),
            team_id: zeta_organization.team_id.to_string(),
            organization_title: zeta_organization.title,
        },
    }
}

pub async fn get_dummy_data(test_env: &TestEnvironment) -> DummyData {
    let (alpha_project, alpha_version) = get_project_alpha(test_env).await;
    let (beta_project, beta_version) = get_project_beta(test_env).await;

    let zeta_organization = get_organization_zeta(test_env).await;
    DummyData {
        project_alpha: DummyProjectAlpha {
            team_id: alpha_project.team.to_string(),
            project_id: alpha_project.id.to_string(),
            project_slug: alpha_project.slug.unwrap(),
            version_id: alpha_version.id.to_string(),
            thread_id: alpha_project.thread_id.to_string(),
            file_hash: alpha_version.files[0].hashes["sha1"].clone(),
        },

        project_beta: DummyProjectBeta {
            team_id: beta_project.team.to_string(),
            project_id: beta_project.id.to_string(),
            project_slug: beta_project.slug.unwrap(),
            version_id: beta_version.id.to_string(),
            thread_id: beta_project.thread_id.to_string(),
            file_hash: beta_version.files[0].hashes["sha1"].clone(),
        },

        organization_zeta: DummyOrganizationZeta {
            organization_id: zeta_organization.id.to_string(),
            team_id: zeta_organization.team_id.to_string(),
            organization_title: zeta_organization.title,
        },
    }
}

pub async fn add_project_alpha(test_env: &TestEnvironment) -> (Project, Version) {
    let (project, versions) = test_env
        .v2
        .add_public_project(
            get_public_project_creation_data("alpha", Some(DummyJarFile::DummyProjectAlpha)),
            USER_USER_PAT,
        )
        .await;
    (project, versions.into_iter().next().unwrap())
}

pub async fn add_project_beta(test_env: &TestEnvironment) -> (Project, Version) {
    // Adds dummy data to the database with sqlx (projects, versions, threads)
    // Generate test project data.
    let jar = DummyJarFile::DummyProjectBeta;
    let json_data = json!(
        {
            "title": "Test Project Beta",
            "slug": "beta",
            "description": "A dummy project for testing with.",
            "body": "This project is not-yet-approved, and versions are draft.",
            "client_side": "required",
            "server_side": "optional",
            "initial_versions": [{
                "file_parts": [jar.filename()],
                "version_number": "1.2.3",
                "version_title": "start",
                "status": "unlisted",
                "requested_status": "unlisted",
                "dependencies": [],
                "game_versions": ["1.20.1"] ,
                "release_channel": "release",
                "loaders": ["fabric"],
                "featured": true
            }],
            "status": "private",
            "requested_status": "private",
            "categories": [],
            "license_id": "MIT"
        }
    );

    // Basic json
    let json_segment = MultipartSegment {
        name: "data".to_string(),
        filename: None,
        content_type: Some("application/json".to_string()),
        data: MultipartSegmentData::Text(serde_json::to_string(&json_data).unwrap()),
    };

    // Basic file
    let file_segment = MultipartSegment {
        name: jar.filename(),
        filename: Some(jar.filename()),
        content_type: Some("application/java-archive".to_string()),
        data: MultipartSegmentData::Binary(jar.bytes()),
    };

    // Add a project.
    let req = TestRequest::post()
        .uri("/v2/project")
        .append_header(("Authorization", USER_USER_PAT))
        .set_multipart(vec![json_segment.clone(), file_segment.clone()])
        .to_request();
    let resp = test_env.call(req).await;

    assert_eq!(resp.status(), 200);

    get_project_beta(test_env).await
}

pub async fn add_organization_zeta(test_env: &TestEnvironment) -> Organization {
    // Add an organzation.
    let req = TestRequest::post()
        .uri("/v2/organization")
        .append_header(("Authorization", USER_USER_PAT))
        .set_json(json!({
            "title": "zeta",
            "description": "A dummy organization for testing with."
        }))
        .to_request();
    let resp = test_env.call(req).await;

    assert_eq!(resp.status(), 200);

    get_organization_zeta(test_env).await
}

pub async fn get_project_alpha(test_env: &TestEnvironment) -> (Project, Version) {
    // Get project
    let req = TestRequest::get()
        .uri("/v2/project/alpha")
        .append_header(("Authorization", USER_USER_PAT))
        .to_request();
    let resp = test_env.call(req).await;
    let project: Project = test::read_body_json(resp).await;

    // Get project's versions
    let req = TestRequest::get()
        .uri("/v2/project/alpha/version")
        .append_header(("Authorization", USER_USER_PAT))
        .to_request();
    let resp = test_env.call(req).await;
    let versions: Vec<Version> = test::read_body_json(resp).await;
    let version = versions.into_iter().next().unwrap();

    (project, version)
}

pub async fn get_project_beta(test_env: &TestEnvironment) -> (Project, Version) {
    // Get project
    let req = TestRequest::get()
        .uri("/v2/project/beta")
        .append_header(("Authorization", USER_USER_PAT))
        .to_request();
    let resp = test_env.call(req).await;
    let project: Project = test::read_body_json(resp).await;

    // Get project's versions
    let req = TestRequest::get()
        .uri("/v2/project/beta/version")
        .append_header(("Authorization", USER_USER_PAT))
        .to_request();
    let resp = test_env.call(req).await;
    let versions: Vec<Version> = test::read_body_json(resp).await;
    let version = versions.into_iter().next().unwrap();

    (project, version)
}

pub async fn get_organization_zeta(test_env: &TestEnvironment) -> Organization {
    // Get organization
    let req = TestRequest::get()
        .uri("/v2/organization/zeta")
        .append_header(("Authorization", USER_USER_PAT))
        .to_request();
    let resp = test_env.call(req).await;
    let organization: Organization = test::read_body_json(resp).await;

    organization
}

impl DummyJarFile {
    pub fn filename(&self) -> String {
        match self {
            DummyJarFile::DummyProjectAlpha => "dummy-project-alpha.jar",
            DummyJarFile::DummyProjectBeta => "dummy-project-beta.jar",
            DummyJarFile::BasicMod => "basic-mod.jar",
            DummyJarFile::BasicModDifferent => "basic-mod-different.jar",
        }
        .to_string()
    }

    pub fn bytes(&self) -> Vec<u8> {
        match self {
            DummyJarFile::DummyProjectAlpha => {
                include_bytes!("../../tests/files/dummy-project-alpha.jar").to_vec()
            }
            DummyJarFile::DummyProjectBeta => {
                include_bytes!("../../tests/files/dummy-project-beta.jar").to_vec()
            }
            DummyJarFile::BasicMod => include_bytes!("../../tests/files/basic-mod.jar").to_vec(),
            DummyJarFile::BasicModDifferent => {
                include_bytes!("../../tests/files/basic-mod-different.jar").to_vec()
            }
        }
    }
}

impl DummyImage {
    pub fn filename(&self) -> String {
        match self {
            DummyImage::SmallIcon => "200x200.png",
        }
        .to_string()
    }

    pub fn extension(&self) -> String {
        match self {
            DummyImage::SmallIcon => "png",
        }
        .to_string()
    }

    pub fn bytes(&self) -> Vec<u8> {
        match self {
            DummyImage::SmallIcon => include_bytes!("../../tests/files/200x200.png").to_vec(),
        }
    }
}