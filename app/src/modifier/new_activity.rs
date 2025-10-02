use anyhow::{Result, anyhow};
use db::ConnCache;
use db::models::{
    ActivityNature, ActivityTxTag, FullTx, NewActivity, NewActivityTx, NewSearch, NewTx, Tag,
};

pub(crate) fn activity_new_tx(tx: &NewTx, tags: &str, conn: &mut impl ConnCache) -> Result<()> {
    let activity_type = ActivityNature::AddTx;

    let new_activity = NewActivity::new(activity_type).insert(conn)?;
    let added_tx = NewActivityTx::new_from_new_tx(tx, new_activity.id).insert(conn)?;

    let mut tag_list = Vec::new();

    if tags.is_empty() {
        tag_list.push("Unknown".to_string());
    } else {
        let split_tags = tags.split(',').collect::<Vec<&str>>();

        for tag in split_tags {
            let trimmed_tag = tag.trim();
            if !trimmed_tag.is_empty() {
                tag_list.push(trimmed_tag.to_string());
            }
        }
    }

    let mut tx_tags = Vec::new();

    for tag in tag_list {
        let tag_id = if let Ok(id) = conn.cache().get_tag_id(&tag) {
            id
        } else {
            let tag = Tag::get_by_name(conn, &tag)?.ok_or(anyhow!(
                "Tag not found but it should have been in the DB by this point"
            ))?;
            tag.id
        };
        let tx_tag = ActivityTxTag::new(added_tx.id, tag_id);
        tx_tags.push(tx_tag);
    }

    ActivityTxTag::insert_batch(tx_tags, conn)?;

    Ok(())
}

pub(crate) fn activity_delete_tx(tx: &FullTx, conn: &mut impl ConnCache) -> Result<()> {
    let activity_type = ActivityNature::DeleteTx;

    let new_activity = NewActivity::new(activity_type).insert(conn)?;

    let added_tx = NewActivityTx::new_from_full_tx(tx, false, new_activity.id).insert(conn)?;

    let mut tag_list = Vec::new();

    for tag in &tx.tags {
        let tag = ActivityTxTag::new(added_tx.id, tag.id);
        tag_list.push(tag);
    }

    ActivityTxTag::insert_batch(tag_list, conn)?;

    Ok(())
}

pub(crate) fn activity_edit_tx(
    old_tx: &FullTx,
    new_tx: &NewTx,
    tags: &str,
    conn: &mut impl ConnCache,
) -> Result<()> {
    let activity_type = ActivityNature::EditTx;

    let new_activity = NewActivity::new(activity_type).insert(conn)?;

    // New one always first!
    let new_tx_activity = NewActivityTx::new_from_new_tx(new_tx, new_activity.id).insert(conn)?;
    let old_tx_activity =
        NewActivityTx::new_from_full_tx(old_tx, false, new_activity.id).insert(conn)?;

    let mut old_tag_list = Vec::new();
    let mut new_tag_list = Vec::new();

    for tag in &old_tx.tags {
        let tag = ActivityTxTag::new(old_tx_activity.id, tag.id);
        old_tag_list.push(tag);
    }

    if tags.is_empty() {
        new_tag_list.push("Unknown".to_string());
    } else {
        let split_tags = tags.split(',').collect::<Vec<&str>>();

        for tag in split_tags {
            let trimmed_tag = tag.trim();
            if !trimmed_tag.is_empty() {
                new_tag_list.push(trimmed_tag.to_string());
            }
        }
    }

    let mut new_tags = Vec::new();

    for tag in new_tag_list {
        let tag_id = if let Ok(id) = conn.cache().get_tag_id(&tag) {
            id
        } else {
            let tag = Tag::get_by_name(conn, &tag)?.ok_or(anyhow!(
                "Tag not found but it should have been in the DB by this point"
            ))?;
            tag.id
        };
        let tx_tag = ActivityTxTag::new(new_tx_activity.id, tag_id);
        new_tags.push(tx_tag);
    }

    ActivityTxTag::insert_batch(new_tags, conn)?;
    ActivityTxTag::insert_batch(old_tag_list, conn)?;

    Ok(())
}

pub(crate) fn activity_search_tx(search_tx: &NewSearch, conn: &mut impl ConnCache) -> Result<()> {
    let activity_type = ActivityNature::SearchTx;

    let new_activity = NewActivity::new(activity_type).insert(conn)?;
    let added_tx = NewActivityTx::new_from_search_tx(search_tx, new_activity.id).insert(conn)?;

    let mut tx_tags = Vec::new();

    if let Some(tags) = &search_tx.tags {
        for tag in tags {
            let tag = ActivityTxTag::new(added_tx.id, *tag);
            tx_tags.push(tag);
        }

        ActivityTxTag::insert_batch(tx_tags, conn)?;
    }

    Ok(())
}

pub(crate) fn activity_swap_position(
    tx_1: &FullTx,
    tx_2: &FullTx,
    conn: &mut impl ConnCache,
) -> Result<()> {
    let activity_type = ActivityNature::PositionSwap;

    let new_activity = NewActivity::new(activity_type).insert(conn)?;

    let tx_1_activity_tx =
        NewActivityTx::new_from_full_tx(tx_1, true, new_activity.id).insert(conn)?;

    let tx_2_activity_tx =
        NewActivityTx::new_from_full_tx(tx_2, true, new_activity.id).insert(conn)?;

    let mut tag_list_tx_1 = Vec::new();
    let mut tag_list_tx_2 = Vec::new();

    for tag in &tx_1.tags {
        let tag = ActivityTxTag::new(tx_1_activity_tx.id, tag.id);
        tag_list_tx_1.push(tag);
    }

    for tag in &tx_2.tags {
        let tag = ActivityTxTag::new(tx_2_activity_tx.id, tag.id);
        tag_list_tx_2.push(tag);
    }

    ActivityTxTag::insert_batch(tag_list_tx_1, conn)?;
    ActivityTxTag::insert_batch(tag_list_tx_2, conn)?;

    Ok(())
}
