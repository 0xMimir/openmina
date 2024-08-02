import {
  BENCHMARKS_WALLETS_CHANGE_FEE,
  BENCHMARKS_WALLETS_CHANGE_TRANSACTION_BATCH,
  BENCHMARKS_WALLETS_CLOSE, BENCHMARKS_WALLETS_GET_ALL_TXS_SUCCESS,
  BENCHMARKS_WALLETS_GET_WALLETS,
  BENCHMARKS_WALLETS_GET_WALLETS_SUCCESS,
  BENCHMARKS_WALLETS_SELECT_WALLET,
  BENCHMARKS_WALLETS_SEND_TX_SUCCESS,
  BENCHMARKS_WALLETS_SEND_TXS,
  BENCHMARKS_WALLETS_TOGGLE_RANDOM_WALLET,
  BENCHMARKS_WALLETS_UPDATE_WALLETS_SUCCESS,
  BenchmarksWalletsActions,
} from '@benchmarks/wallets/benchmarks-wallets.actions';
import {
  BenchmarksWallet,
  BenchmarksWalletTransactionStatus,
} from '@shared/types/benchmarks/wallets/benchmarks-wallet.type';
import { BenchmarksWalletTransaction } from '@shared/types/benchmarks/wallets/benchmarks-wallet-transaction.type';
import { hasValue, lastItem, ONE_BILLION, toReadableDate } from '@openmina/shared';
import { BenchmarksWalletsState } from '@benchmarks/wallets/benchmarks-wallets.state';
import { getTimeFromMemo } from '@shared/helpers/transaction.helper';

const initialState: BenchmarksWalletsState = {
  wallets: [],
  txsToSend: [],
  blockSending: false,
  txSendingBatch: undefined,
  sentTransactions: {
    success: 0,
    fail: 0,
  },
  sentTxCount: 0,
  randomWallet: true,
  activeWallet: undefined,
  sendingFee: 1,
};

export function reducer(state: BenchmarksWalletsState = initialState, action: BenchmarksWalletsActions): BenchmarksWalletsState {
  switch (action.type) {

    case BENCHMARKS_WALLETS_GET_WALLETS_SUCCESS: {
      const wallets = action.payload.map((wallet, i: number) => ({
        successTx: 0,
        failedTx: 0,
        ...state.wallets[i],
        ...wallet,
      }));
      return {
        ...state,
        wallets,
        blockSending: false,
        txSendingBatch: !hasValue(state.txSendingBatch) ? action.payload.length : state.txSendingBatch,
        activeWallet: wallets[0],
      };
    }

    case BENCHMARKS_WALLETS_UPDATE_WALLETS_SUCCESS: {
      return {
        ...state,
        wallets: state.wallets.map((wallet: BenchmarksWallet, i: number) => ({
          ...wallet,
          ...action.payload[i],
        })),
      };
    }

    case BENCHMARKS_WALLETS_CHANGE_TRANSACTION_BATCH: {
      return {
        ...state,
        txSendingBatch: action.payload,
      };
    }

    case BENCHMARKS_WALLETS_GET_WALLETS: {
      return {
        ...state,
        blockSending: true,
      };
    }

    case BENCHMARKS_WALLETS_TOGGLE_RANDOM_WALLET: {
      return {
        ...state,
        randomWallet: !state.randomWallet,
      };
    }

    case BENCHMARKS_WALLETS_SEND_TXS: {
      let txsToSend: BenchmarksWalletTransaction[];
      if (state.randomWallet) {
        txsToSend = state.wallets
          .slice(0, state.txSendingBatch)
          .map((wallet: BenchmarksWallet, i: number) => {
            const nonce = getNonceForWallet(wallet, state).toString();
            const counter = state.sentTxCount + i;
            const memo = 'S.T.' + Date.now() + ',' + (counter + 1) + ',' + localStorage.getItem('browserId');
            const payment = {
              from: wallet.publicKey,
              nonce,
              to: getRandomReceiver(wallet, state.wallets),
              fee: (state.sendingFee * ONE_BILLION).toString(),
              amount: '2000000000',
              memo,
              validUntil: '4294967295',
            };

            return {
              ...payment,
              privateKey: wallet.privateKey,
            };
          });
      } else {
        const wallet = state.activeWallet;
        let nonce = getNonceForWallet(wallet, state);

        txsToSend = Array(state.txSendingBatch).fill(void 0).map((_, i: number) => {
          const counter = state.sentTxCount + i;
          const memo = 'S.T.' + Date.now() + ',' + (counter + 1) + ',' + localStorage.getItem('browserId');
          const payment = {
            from: wallet.publicKey,
            nonce: nonce.toString(),
            to: state.wallets[i].publicKey,
            fee: (state.sendingFee * ONE_BILLION).toString(),
            amount: '2000000000',
            memo,
            validUntil: '4294967295',
          };
          nonce++;

          return {
            ...payment,
            privateKey: wallet.privateKey,
          };
        });
      }

      return {
        ...state,
        txsToSend,
        wallets: state.wallets.map((w: BenchmarksWallet) => {
          const transactionFromThisWallet = txsToSend.find(tx => tx.from === w.publicKey);
          if (!transactionFromThisWallet) {
            return w;
          }
          return {
            ...w,
            lastTxTime: getTimeFromMemo(transactionFromThisWallet.memo),
            lastTxMemo: transactionFromThisWallet.memo,
            lastTxStatus: BenchmarksWalletTransactionStatus.SENDING,
          };
        }),
      };
    }

    case BENCHMARKS_WALLETS_SEND_TX_SUCCESS: {
      return {
        ...state,
        txsToSend: [],
        wallets: state.wallets.map((w: BenchmarksWallet) => {
          const transactionsFromThisWallet = action.payload.transactions.filter(tx => tx.from === w.publicKey);
          if (transactionsFromThisWallet.length === 0) {
            return w;
          }
          return {
            ...w,
            lastTxCount: lastItem(transactionsFromThisWallet).memo.split(',')[1],
            lastTxStatus: action.payload.error ? BenchmarksWalletTransactionStatus.ERROR : BenchmarksWalletTransactionStatus.GENERATED,
            successTx: w.successTx + (!action.payload.error ? transactionsFromThisWallet.length : 0),
            failedTx: w.failedTx + (action.payload.error ? transactionsFromThisWallet.length : 0),
            lastTxTime: lastItem(transactionsFromThisWallet).dateTime,
            lastTxMemo: lastItem(transactionsFromThisWallet).memo.replace('S.T.', ''),
            errorReason: action.payload.error?.message,
          };
        }),
        sentTransactions: {
          success: state.sentTransactions.success + (!action.payload.error ? action.payload.transactions.length : 0),
          fail: state.sentTransactions.fail + (action.payload.error ? action.payload.transactions.length : 0),
        },
      };
    }

    case BENCHMARKS_WALLETS_GET_ALL_TXS_SUCCESS: {
      const allTxs = [...action.payload.mempoolTxs, ...action.payload.includedTxs];
      return {
        ...state,
        wallets: state.wallets.map((w: BenchmarksWallet) => {
          const transactionsFromThisWallet = allTxs.filter(tx => tx.sender === w.publicKey);
          if (transactionsFromThisWallet.length === 0) {
            return w;
          }
          const lastTransaction = transactionsFromThisWallet.reduce((acc, tx) => {
            if (parseInt(tx.memo.split(',')[1], 10) > parseInt(acc.memo.split(',')[1], 10)) {
              return tx;
            }
            return acc;
          });
          return {
            ...w,
            lastTxMemo: lastTransaction.memo.replace('S.T.', ''),
            lastTxStatus: action.payload.mempoolTxs.includes(lastTransaction) ? BenchmarksWalletTransactionStatus.GENERATED : BenchmarksWalletTransactionStatus.INCLUDED,
            lastTxCount: lastTransaction.memo.split(',')[1],
            lastTxTime: getTimeFromMemo(lastTransaction.memo),
            successTx: allTxs.filter(tx => tx.sender === w.publicKey && tx.memo.includes('S.T.')).length,
          };
        }),
        sentTransactions: {
          ...state.sentTransactions,
          success: allTxs.filter(tx => tx.memo.includes('S.T.')).length,
        },
      };
    }

    case BENCHMARKS_WALLETS_SELECT_WALLET: {
      return {
        ...state,
        activeWallet: action.payload,
      };
    }

    case BENCHMARKS_WALLETS_CHANGE_FEE: {
      return {
        ...state,
        sendingFee: action.payload,
      };
    }

    case BENCHMARKS_WALLETS_CLOSE: {
      return {
        ...initialState,
        sentTxCount: state.sentTxCount,
      };
    }

    default:
      return state;
  }
}

function getRandomReceiver(currentWallet: BenchmarksWallet, wallets: BenchmarksWallet[]): string {
  const index = Math.floor(Math.random() * wallets.length);
  if (wallets[index].publicKey === currentWallet.publicKey) {
    return getRandomReceiver(currentWallet, wallets);
  }
  return wallets[index].publicKey;
}

function getNonceForWallet(wallet: BenchmarksWallet, state: BenchmarksWalletsState): number {
  // const txsInMempool = state.mempoolTxs.filter(tx => tx.from === wallet.publicKey).map(tx => tx.nonce);
  return Math.max(wallet.nonce, 0);
}