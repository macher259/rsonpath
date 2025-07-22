use rsonpath_benchmarks::prelude::*;

fn az_shallow_tenant_ids(c: &mut Criterion) -> Result<(), BenchmarkError> {
    let benchset = Benchset::new("az_tenant::shallow_ids", dataset::az_tenants())?
        .add_all_rsonpath_streaming("$[*].tenantId")?
        .finish();

    benchset.run(c);

    Ok(())
}

fn az_recursive_tenant_ids(c: &mut Criterion) -> Result<(), BenchmarkError> {
    let benchset = Benchset::new("az_tenants::recursive_ids", dataset::az_tenants())?
        .add_all_rsonpath_streaming("$..tenantId")?
        .finish();

    benchset.run(c);

    Ok(())
}

fn az_first_ten_tenant_ids(c: &mut Criterion) -> Result<(), BenchmarkError> {
    let benchset = Benchset::new("az_tenants::first_ten_tenant_ids", dataset::az_tenants())?
        .add_all_rsonpath_streaming("$[:10].tenantId")?
        .finish();

    benchset.run(c);

    Ok(())
}

fn az_tenant_17(c: &mut Criterion) -> Result<(), BenchmarkError> {
    let benchset = Benchset::new("az_tenants::tenant_17", dataset::az_tenants())?
        .add_all_rsonpath_streaming("$[17]")?
        .finish();

    benchset.run(c);

    Ok(())
}

fn az_tenant_last(c: &mut Criterion) -> Result<(), BenchmarkError> {
    let benchset = Benchset::new("az_tenants::tenant_last", dataset::az_tenants())?
        .add_all_rsonpath_streaming("$[83]")?
        .finish();

    benchset.run(c);

    Ok(())
}

fn az_every_other_tenant(c: &mut Criterion) -> Result<(), BenchmarkError> {
    let benchset = Benchset::new("az_tenants::every_other_tenant", dataset::az_tenants())?
        .add_all_rsonpath_streaming("$[::2]")?
        .finish();

    benchset.run(c);

    Ok(())
}

pub fn all_first_index(c: &mut Criterion) -> Result<(), BenchmarkError> {
    let benchset = Benchset::new("all_first_index", dataset::twitter())?
        .add_all_rsonpath_streaming("$..[0]")?
        .finish();

    benchset.run(c);

    Ok(())
}

pub fn ast_deepest(c: &mut Criterion) -> Result<(), BenchmarkError> {
    let benchset = Benchset::new("ast::deepest", dataset::ast())?
        .add_all_rsonpath_streaming("$..*..*..*..*..*..*..*..*..*..*..*..*..*..*..*..*..*..*..*..*..*..*..*..*..*..*..*..*..*..*..*..*..*..*..*..*..*..*..*..*..*..*..*..*..*..*..*..*..*..*..*..*..*..*..*..*..*..*..*..*..*..*..*..*..*..*..*..*..*..*..*..*..*..*..*..*..*..*..*..*..*..*..*..*..*..*..*..*..*..*..*..*..*..*..*..*..*..*..*..*..*")?
        .finish();

    benchset.run(c);

    Ok(())
}

pub fn twitter_metadata_direct(c: &mut Criterion) -> Result<(), BenchmarkError> {
    let benchset = Benchset::new("twitter::metadata::direct", dataset::twitter())?
        .add_all_rsonpath_streaming("$.search_metadata.count")?
        .finish();

    benchset.run(c);

    Ok(())
}

pub fn twitter_metadata_descendant(c: &mut Criterion) -> Result<(), BenchmarkError> {
    let benchset = Benchset::new("twitter::metadata::descendant", dataset::twitter())?
        .add_all_rsonpath_streaming("$..count")?
        .finish();

    benchset.run(c);

    Ok(())
}

pub fn inner_array(c: &mut Criterion) -> Result<(), BenchmarkError> {
    let benchset = Benchset::new("inner_array", dataset::ast())?
        .add_all_rsonpath_streaming("$..inner[0]")?
        .finish();

    benchset.run(c);

    Ok(())
}

pub fn gh_events_all_sha(c: &mut Criterion) -> Result<(), BenchmarkError> {
    let benchset = Benchset::new("gh_events::sha", dataset::gh_events())?
        .add_all_rsonpath_streaming("$..sha")?
        .finish();

    benchset.run(c);

    Ok(())
}

pub fn gh_events_pr_url(c: &mut Criterion) -> Result<(), BenchmarkError> {
    let benchset = Benchset::new("gh_events::pr", dataset::gh_events())?
        .add_all_rsonpath_streaming("$.payload.payload.pull_request.url")?
        .finish();

    benchset.run(c);

    Ok(())
}

pub fn gh_events_big(c: &mut Criterion) -> Result<(), BenchmarkError> {
    let benchset = Benchset::new("gh_events_big::sha", dataset::gh_events_big())?
        .add_all_rsonpath_streaming("$..sha")?
        .finish();

    benchset.run(c);

    Ok(())
}

pub fn gh_events_pr_url_big(c: &mut Criterion) -> Result<(), BenchmarkError> {
    let benchset = Benchset::new("gh_events_big::pr", dataset::gh_events_big())?
        .add_all_rsonpath_streaming("$.payload.payload.pull_request.url")?
        .finish();

    benchset.run(c);

    Ok(())
}

benchsets!(
    main_streaming_benches,
    az_shallow_tenant_ids,
    az_recursive_tenant_ids,
    az_first_ten_tenant_ids,
    az_tenant_17,
    az_tenant_last,
    az_every_other_tenant,
    all_first_index,
    ast_deepest,
    twitter_metadata_direct,
    twitter_metadata_descendant,
    inner_array,
    gh_events_all_sha,
    gh_events_pr_url,
    gh_events_big,
    gh_events_pr_url_big
);
