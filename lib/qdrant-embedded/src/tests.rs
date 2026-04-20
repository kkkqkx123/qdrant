#[cfg(test)]
mod tests {
    use super::*;
    use api::{VectorsConfig, VectorParams, Distance, PointStruct, PointId, SearchRequest};

    #[tokio::test]
    async fn test_create_and_list_collections() {
        let temp = tempfile::tempdir().unwrap();
        let config = EmbeddedConfig::builder()
            .storage_path(temp.path())
            .build();

        let qdrant = QdrantEmbedded::new(config).unwrap();

        let vectors_config = VectorsConfig {
            config: VectorParams {
                size: 10,
                distance: Distance::Cosine,
                ..Default::default()
            },
            ..Default::default()
        };

        qdrant.create_collection("test".to_string(), vectors_config).await.unwrap();

        let collections = qdrant.list_collections().await.unwrap();
        assert_eq!(collections.len(), 1);
        assert_eq!(collections[0], "test");
    }

    #[tokio::test]
    async fn test_upsert_and_search() {
        let temp = tempfile::tempdir().unwrap();
        let config = EmbeddedConfig::builder()
            .storage_path(temp.path())
            .build();

        let qdrant = QdrantEmbedded::new(config).unwrap();

        // Create collection
        let vectors_config = VectorsConfig {
            config: VectorParams {
                size: 2,
                distance: Distance::Cosine,
                ..Default::default()
            },
            ..Default::default()
        };
        qdrant.create_collection("test".to_string(), vectors_config).await.unwrap();

        // Upsert points
        let points = vec![
            PointStruct {
                id: PointId::Num(1),
                vector: vec![1.0, 0.0].into(),
                payload: None,
            },
            PointStruct {
                id: PointId::Num(2),
                vector: vec![0.0, 1.0].into(),
                payload: None,
            },
        ];
        qdrant.upsert_points("test".to_string(), points).await.unwrap();

        // Search
        let search_request = SearchRequest {
            vector: vec![1.0, 0.0].into(),
            limit: 2,
            ..Default::default()
        };
        let results = qdrant.search_points("test".to_string(), search_request).await.unwrap();

        assert_eq!(results.result.len(), 2);
        assert_eq!(results.result[0].id, PointId::Num(1));
    }
}
