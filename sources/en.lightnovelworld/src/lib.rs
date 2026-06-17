#![no_std]
use aidoku::{
	AidokuError, Chapter, DeepLinkHandler, DeepLinkResult, FilterValue, Listing,
	ListingProvider, Manga, MangaPageResult, Page, Result, Source,
	alloc::{String, Vec, format},
	imports::net::Request,
	prelude::*,
};

const BASE_URL: &str = "https://lightnovelworld.org";

struct Lightnovelworld;

impl Source for Lightnovelworld {
	fn new() -> Self {
		Self
	}

	fn get_search_manga_list(
		&self,
		query: Option<String>,
		page: i32,
		_filters: Vec<FilterValue>,
	) -> Result<MangaPageResult> {
		let url = match query {
			Some(q) => format!("{}/search/?title={}&page={}", BASE_URL, q, page),
			None => format!("{}/genre-all/?page={}", BASE_URL, page),
		};
		parse_manga_list(&url)
	}

	fn get_manga_update(
		&self,
		manga: Manga,
		needs_details: bool,
		needs_chapters: bool,
	) -> Result<Manga> {
		if !needs_details && !needs_chapters {
			return Ok(manga);
		}
		let url = format!("{}{}", BASE_URL, manga.key);
		let html = Request::get(&url)?.html()?;
		let mut m = manga.clone();
		if needs_details {
			if let Some(title) = html.select_first("h1").and_then(|e| e.text()) {
				m.title = title;
			}
			if let Some(desc) = html.select_first(".summary").and_then(|e| e.text()) {
				m.description = Some(desc);
			}
			if let Some(cover) = html.select_first(".cover img").and_then(|e| e.attr("src")) {
				m.cover = Some(if cover.starts_with("http") {
					cover
				} else {
					format!("{}{}", BASE_URL, cover)
				});
			}
		}
		if needs_chapters {
			let mut chapters = Vec::new();
			let mut page = 1;
			loop {
				let chapters_url = format!("{}{}/chapters/?page={}", BASE_URL, manga.key, page);
				let chapters_html = Request::get(&chapters_url)?.html()?;
				let mut found = false;
				if let Some(items) = chapters_html.select("li a[href*='/chapter/']") {
					for item in items {
						let key = item.attr("href").unwrap_or_default();
						let title = item.attr("title")
							.or_else(|| item.text())
							.unwrap_or_default();
						if !key.is_empty() {
							chapters.push(Chapter {
								key,
								title: Some(title),
								..Default::default()
							});
							found = true;
						}
					}
				}
				if !found {
					break;
				}
				page += 1;
			}
			chapters.reverse();
			m.chapters = Some(chapters);
		}
		Ok(m)
	}

	fn get_page_list(&self, _manga: Manga, chapter: Chapter) -> Result<Vec<Page>> {
		let url = format!("{}{}", BASE_URL, chapter.key);
		let html = Request::get(&url)?.html()?;
		let content = html
			.select_first(".chapter-content")
			.or_else(|| html.select_first("#chapter-content"))
			.and_then(|e| e.text())
			.unwrap_or_default();
		Ok(Vec::from([Page {
			content: aidoku::PageContent::Text(content),
			..Default::default()
		}]))
	}
}

impl ListingProvider for Lightnovelworld {
	fn get_manga_list(&self, _listing: Listing, page: i32) -> Result<MangaPageResult> {
		let url = format!("{}/genre-all/?page={}", BASE_URL, page);
		parse_manga_list(&url)
	}
}

fn parse_manga_list(url: &str) -> Result<MangaPageResult> {
	let html = Request::get(url)?.html()?;
	let mut entries = Vec::new();
	if let Some(items) = html.select("li.novel-item a") {
		for item in items {
			let title = item.attr("title").unwrap_or_default();
			let key = item.attr("href").unwrap_or_default();
			let cover = item.select_first("img").and_then(|e| {
				e.attr("data-src").or_else(|| e.attr("src")).map(|src| {
					if src.starts_with("http") {
						src
					} else {
						format!("{}{}", BASE_URL, src)
					}
				})
			}).unwrap_or_default();
			if !key.is_empty() && !title.is_empty() {
				entries.push(Manga {
					key,
					title,
					cover: Some(cover),
					..Default::default()
				});
			}
		}
	}
	Ok(MangaPageResult { entries, has_next_page: true })
}

impl DeepLinkHandler for Lightnovelworld {
	fn handle_deep_link(&self, _url: String) -> Result<Option<DeepLinkResult>> {
		Err(AidokuError::Unimplemented)
	}
}

register_source!(Lightnovelworld, ListingProvider, DeepLinkHandler);