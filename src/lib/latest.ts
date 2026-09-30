/**
 * Ne retient que la réponse de la requête la plus récente : une réponse lente
 * ne peut plus écraser le résultat d'une saisie postérieure.
 */
export class LatestRequest {
  private current = 0;

  /** Démarre une requête ; la fonction renvoyée dit si elle est toujours la plus récente. */
  begin(): () => boolean {
    const id = ++this.current;
    return () => id === this.current;
  }

  /** Rend périmées toutes les requêtes en cours. */
  invalidate(): void {
    this.current++;
  }
}
