import { NgModule } from '@angular/core';

import { DashboardRouting } from '@dashboard/dashboard.routing';
import { DashboardPeersComponent } from '@dashboard/dashboard-peers/dashboard-peers.component';
import { DashboardNodeComponent } from '@dashboard/dashboard-node/dashboard-node.component';
import { DashboardComponent } from '@dashboard/dashboard.component';
import { SharedModule } from '@shared/shared.module';
import { DashboardPeersTableComponent } from '@dashboard/dashboard-peers-table/dashboard-peers-table.component';
import { DashboardBlockHeightComponent } from '@dashboard/dashboard-block-height/dashboard-block-height.component';
import { DashboardReceivedComponent } from '@dashboard/dashboard-received/dashboard-received.component';
import { EffectsModule } from '@ngrx/effects';
import { DashboardEffects } from '@dashboard/dashboard.effects';
import { LoadingSpinnerComponent } from '@shared/loading-spinner/loading-spinner.component';
import { CopyComponent } from '@openmina/shared';
import { DashboardNetworkComponent } from './dashboard-network/dashboard-network.component';
import { DashboardLedgerComponent } from './dashboard-ledger/dashboard-ledger.component';
import {
  DashboardTransitionFrontierComponent,
} from './dashboard-transition-frontier/dashboard-transition-frontier.component';
import { DashboardBlocksSyncComponent } from './dashboard-blocks-sync/dashboard-blocks-sync.component';
import { DashboardErrorsComponent } from './dashboard-errors/dashboard-errors.component';
import {
  DashboardPeersMinimalTableComponent,
} from './dashboard-peers-minimal-table/dashboard-peers-minimal-table.component';


@NgModule({
  declarations: [
    DashboardComponent,
    DashboardPeersComponent,
    DashboardNodeComponent,
    DashboardPeersTableComponent,
    DashboardBlockHeightComponent,
    DashboardReceivedComponent,
    DashboardNetworkComponent,
    DashboardLedgerComponent,
    DashboardTransitionFrontierComponent,
    DashboardBlocksSyncComponent,
    DashboardErrorsComponent,
    DashboardPeersMinimalTableComponent,
  ],
  imports: [
    SharedModule,
    DashboardRouting,
    EffectsModule.forFeature(DashboardEffects),
    LoadingSpinnerComponent,
    CopyComponent,
  ],
})
export class DashboardModule {}
